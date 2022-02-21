use std::{
    ffi::{OsStr, OsString},
    io::{Read, Write},
    process::{Child, Command, Stdio},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use spdlog::warn;

use crate::core::CoreTool;

use core_protobuf::v2ray_proto::{OutboundObject, V2RayConfig};

#[derive(Debug)]
pub struct Shadowsocks {
    config: String,
    child_process: Option<Child>,
    path: OsString,
    name: String,
    version: semver::Version,
}

impl Shadowsocks {
    pub fn new<S: AsRef<OsStr> + ?Sized>(path: &S) -> Result<Self> {
        let output = Self::spawn_version_process(path.as_ref())?;

        let (name, version) = Self::get_name_and_version(output)?;

        let semver_version = semver::Version::parse(&version)?;

        let path = path.into();

        Ok(Self {
            name,
            version: semver_version,
            path,
            config: String::new(),
            child_process: None,
        })
    }
    fn spawn_version_process(path: &OsStr) -> Result<Child> {
        let mut process = Command::new(path)
            .arg("--version")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut stdin = process
            .stdin
            .take()
            .ok_or_else(|| anyhow!("No ChildStdin"))?;
        stdin.write_all(b"}{")?;
        Ok(process)
    }

    fn get_name_and_version(mut child: Child) -> Result<(String, String)> {
        child.wait()?;
        let mut stdout = match child.stdout {
            Some(out) => out,
            None => return Err(anyhow!("No child stdout")),
        };
        let mut output = String::new();
        stdout.read_to_string(&mut output)?;

        let core_info = output
            .lines()
            .next()
            .ok_or_else(|| anyhow!("Failed to fetch core name&version"))?;
        let mut info_split = core_info.split(' ');
        let name = info_split.next().unwrap_or("").to_string();
        let version = info_split.next().unwrap_or("").to_string();
        Ok((name, version))
    }

    fn generate_config(
        node_data: &core_data::NodeData,
        inbounds: &config_manager::Inbounds,
    ) -> Result<String> {
        let mut config = String::new();

        if inbounds.http.is_some() {
            warn!("Shadowsocks currently don't have http inbounds.");
        }
        Self::set_inbounds(inbounds, &mut config)?;

        let json = serialize_value(node_data)?;

        match node_data.url.split("://").next() {
            Some(protocol) => {
                if protocol != "ss" {
                    return Err(anyhow!("Protocol Error:"));
                }
                config.push_str(&format!(" --server-url {}", node_data.url));
            }
            None => {
                add_outbound(json, &mut config)?;
            }
        };

        Ok(config)
    }
    fn set_inbounds(
        inbounds: &config_manager::Inbounds,
        config: &mut String,
    ) -> Result<(), anyhow::Error> {
        let socks = inbounds
            .socks5
            .as_ref()
            .ok_or_else(|| anyhow!("Socks inbound not found"))?;
        config.push_str(&format!(" --local-addr {}:{}", socks.listen, socks.port));
        Ok(())
    }
}

fn add_outbound(json: serde_json::Value, config: &mut String) -> Result<(), anyhow::Error> {
    let outbound = get_outbound(&json)?;
    let protocol = protocol_str(&outbound)?;
    if protocol != "shadowsocks" {
        warn!("Protocol Error:")
    };
    let server = get_server(&outbound)?;
    let address = server
        .get("address")
        .ok_or_else(|| anyhow!("No Address"))?
        .as_str()
        .ok_or_else(|| anyhow!("Address should be a String"))?;
    let port = server
        .get("port")
        .ok_or_else(|| anyhow!("No Port"))?
        .as_u64()
        .ok_or_else(|| anyhow!("Port should be an integer"))?;
    let method = server
        .get("method")
        .ok_or_else(|| anyhow!("No Method"))?
        .as_str()
        .ok_or_else(|| anyhow!("Method should be a String"))?;
    let password = server
        .get("password")
        .ok_or_else(|| anyhow!("No Password"))?
        .as_str()
        .ok_or_else(|| anyhow!("Password should be a String"))?;
    config.push_str(&format!(
        " --server-addr {}:{} --encrypt-method {} --password {}",
        address, port, method, password
    ));
    Ok(())
}

fn get_server(outbound: &serde_json::Value) -> Result<&serde_json::Value, anyhow::Error> {
    Ok(outbound
        .get("settings")
        .ok_or_else(|| anyhow!("No shadowsocks settings"))?
        .get("servers")
        .ok_or_else(|| anyhow!("No shadowsocks servers"))?
        .as_array()
        .ok_or_else(|| anyhow!("Servers should be an array"))?
        .get(0)
        .ok_or_else(|| anyhow!("Servers array is empty"))?)
}

fn get_outbound(json: &serde_json::Value) -> Result<&serde_json::Value, anyhow::Error> {
    Ok(json
        .get("outbounds")
        .ok_or_else(|| anyhow!("No Outbounds"))?
        .as_array()
        .ok_or_else(|| anyhow!("Outbounds should be an array"))?
        .get(0)
        .ok_or_else(|| anyhow!("Outbounds empty"))?)
}

fn protocol_str(outbound: &serde_json::Value) -> Result<&str, anyhow::Error> {
    Ok(outbound
        .get("protocol")
        .ok_or_else(|| anyhow!("No protocol specified"))?
        .as_str()
        .ok_or_else(|| anyhow!("Protocol should be a string"))?)
}

impl CoreTool for Shadowsocks {
    fn run(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(anyhow!("Core is running"));
        }

        let mut config: Vec<_> = self.config.split(' ').collect();
        config.retain(|s| !s.is_empty());

        let child = Command::new(&self.path)
            .args(config)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()?;

        self.child_process = Some(child);
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if !self.is_running() {
            return Err(anyhow!("Core not runnning"));
        }

        let mut child = self.child_process.take().unwrap();

        child.kill()?;
        child.wait()?;

        Ok(())
    }

    fn is_running(&mut self) -> bool {
        if self.child_process.is_none() {
            false
        } else {
            let child = self.child_process.as_mut().unwrap();
            matches!(child.try_wait(), Ok(None))
        }
    }

    fn set_config(&mut self, config: String) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update_config(&mut self, config: String) -> Result<()> {
        self.set_config(config)?;
        if self.is_running() {
            self.restart()?;
        }
        Ok(())
    }

    fn get_stdout(&mut self) -> Option<std::process::ChildStdout> {
        self.child_process
            .as_mut()
            .and_then(|child| child.stdout.take())
    }

    fn get_version(&self) -> semver::Version {
        self.version.clone()
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_config(&self) -> &str {
        &self.config
    }

    fn set_config_by_node_and_inbounds(
        &mut self,
        node_data: &core_data::NodeData,
        inbounds: &config_manager::Inbounds,
    ) -> Result<()> {
        let config = Self::generate_config(node_data, inbounds)?;

        self.set_config(config)
    }
}

fn serialize_value(node_data: &core_data::NodeData) -> Result<serde_json::Value, anyhow::Error> {
    let mut node_config = V2RayConfig::default();
    Ok(if node_data.url.contains("://") {
        let mut outbound = json_to_outbound(&node_data.raw)?;

        if outbound.tag.is_empty() {
            outbound.tag = "PROXY".to_string();
        }

        node_config.outbounds.push(outbound);

        config_to_json(&node_config, "")?
    } else {
        config_to_json(&node_config, &node_data.raw)?
    })
}

fn json_to_outbound(json_str: &str) -> Result<OutboundObject, serde_json::Error> {
    serde_json::from_str(json_str)
}

fn config_to_json(origin_config: &V2RayConfig, outbound_raw: &str) -> Result<serde_json::Value> {
    let mut root = serde_json::to_value(&origin_config)?;

    let fix_format = |root: &mut serde_json::Value, keys: Vec<&'static str>| {
        keys.into_iter().for_each(|key| {
            if let serde_json::Value::Array(xbounds) = &mut root[key] {
                xbounds.iter_mut().for_each(|xbound| {
                    let protocol = xbound["protocol"]
                        .as_str()
                        .unwrap_or("null")
                        .replace('-', "_");

                    let setting = &mut xbound["settings"][&protocol];
                    xbound["settings"] = setting.clone();
                });
            }
        });
    };

    if outbound_raw.is_empty() {
        fix_format(&mut root, vec!["outbounds"]);
    } else {
        let outbound = serde_json::Value::from_str(outbound_raw)?;

        if !outbound.is_null() {
            root["outbounds"][0] = outbound;
        }
    }

    Ok(root)
}

#[cfg(test)]
mod tests {

    use std::{thread::sleep, time::Duration};

    use super::*;
    use anyhow::Result;
    #[test]
    fn test_core_version() -> Result<()> {
        let core = Shadowsocks::new("sslocal-rust")?;
        dbg!(core.name, core.version);
        Ok(())
    }
    #[test]
    fn test_core_run() -> Result<()> {
        let mut core = Shadowsocks::new("sslocal-rust")?;

        assert_eq!(false, core.is_running());
        core.set_config("--help".to_string())?;
        core.run()?;
        sleep(Duration::from_millis(500));
        assert_eq!(false, core.is_running());

        core.set_config("--local-addr 127.0.0.1:55342 --server-url ss://YWVzLTI1Ni1nY206cGFzc3dvcmQ@127.0.0.1:8388/%3Bserver%3Btls%3Bhost%3Dgithub.com".to_string())?;

        core.run()?;
        assert_eq!(true, core.is_running());

        core.restart()?;
        assert_eq!(true, core.is_running());
        sleep(Duration::from_millis(500));

        Ok(())
    }
}
