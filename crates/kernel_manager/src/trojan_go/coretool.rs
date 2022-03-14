use std::{
    ffi::{OsStr, OsString},
    io::{Read, Write},
    process::{Child, Command, Stdio},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use spdlog::warn;

use crate::core::CoreTool;

#[derive(Debug)]
pub struct TrojanGo {
    config: String,
    child_process: Option<Child>,
    path: OsString,
    name: String,
    version: String,
}

impl TrojanGo {
    pub fn new<S: AsRef<OsStr> + ?Sized>(path: &S) -> Result<Self> {
        let output = Self::spawn_version_process(path.as_ref())?;

        let (name, mut version) = Self::get_name_and_version(output)?;
        version.retain(|c| !c.is_alphabetic());

        let path = path.into();

        Ok(Self {
            name,
            version,
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
            warn!("TrojanGo currently don't have http inbounds.");
        }
        Self::set_inbounds(inbounds, &mut config)?;

        let mut split_url = node_data.url.split("://");
        let protocol = split_url.next().unwrap_or("");
        let content = split_url.next().unwrap_or("");
        match protocol {
            "trojan" | "trojan-go" => {
                config.push_str(&format!(" -url trojan-go://{}", content));
            }
            "" => {
                let json = serde_json::Value::from_str(&node_data.raw)?;
                add_outbound(&json, &mut config, &node_data.name)?;
            }
            _ => {
                return Err(anyhow!("Protocol Error: {}", protocol));
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
        config.push_str(&format!(
            " -url-option listen={}:{}",
            socks.listen, socks.port
        ));
        Ok(())
    }
}

fn add_outbound(
    json: &serde_json::Value,
    config: &mut String,
    name: &str,
) -> Result<(), anyhow::Error> {
    let protocol = protocol_str(json)?;
    match protocol {
        "trojan" => add_trojan_outbound(json, protocol, config, name),
        "trojan-go" => add_trojan_go_outbound(),
        _ => Err(anyhow!("Protocol Error: {}", protocol)),
    }
}

fn add_trojan_go_outbound() -> Result<(), anyhow::Error> {
    Err(anyhow!("Trojan-go unimplemented"))
}

fn add_trojan_outbound(
    outbound: &serde_json::Value,
    protocol: &str,
    config: &mut String,
    name: &str,
) -> Result<(), anyhow::Error> {
    let server = get_server(outbound, protocol)?;
    let address = get_address(server)?;
    let port = get_port(server)?;
    let password = get_password(server)?;
    let stream_settings = get_stream_settings(outbound)?;
    let network = get_network(stream_settings)?;
    let mut flag = false;
    let mut url = format!(" -url trojan-go://{}@{}:{}/?", password, address, port);
    if let Ok(settings) = get_tls_settings(stream_settings) {
        let sni = get_sni(settings)?;
        flag = true;
        url.push_str(&format!("sni={}", sni));
    }
    match network {
        "original" => {}
        "ws" => {
            let path;
            let mut host = "";
            if let Ok(settings) = get_ws_settings(stream_settings) {
                path = get_path(settings)?;
                host = get_ws_host(settings)?;
            } else {
                path = "/";
            };
            if flag {
                url.push('&');
            }
            url.push_str(&format!("type=ws&path={}", path));
            if !host.is_empty() {
                url.push_str(&format!("&host={}", host))
            }
        }
        "h2" => {
            let settings = get_h2_settings(stream_settings)?;
            let path = get_path(settings)?;
            let hosts = get_h2_hosts(settings)?;
            let mut host_result = String::new();

            for host in hosts {
                host_result.push_str(host);
                host_result.push(',');
            }

            let host_result = &host_result[0..host_result.len() - 1];

            if flag {
                url.push('&');
            }
            url.push_str(&format!(
                "type=h2&path={}&host={}",
                path,
                urlencoding::encode(host_result),
            ));
        }
        _ => {
            return Err(anyhow!("No Such network: {}", network));
        }
    }
    url.push_str(&format!("#{}", name));
    config.push_str(&url);
    Ok(())
}
fn get_h2_hosts(settings: &serde_json::Value) -> Result<Vec<&str>, anyhow::Error> {
    let option_hosts = settings
        .get("host")
        .unwrap_or(&serde_json::Value::Null)
        .as_array();
    let mut result = Vec::new();
    if let Some(hosts) = option_hosts {
        for host in hosts {
            let host = host
                .as_str()
                .ok_or_else(|| anyhow!("Host element should be a String"))?;
            result.push(host);
        }
    }

    Ok(result)
}
fn get_h2_settings(
    stream_settings: &serde_json::Value,
) -> Result<&serde_json::Value, anyhow::Error> {
    stream_settings
        .get("httpSettings")
        .ok_or_else(|| anyhow!("No httpSettings"))
}
fn get_ws_host(settings: &serde_json::Value) -> Result<&str, anyhow::Error> {
    Ok(settings
        .get("headers")
        .map(|headers| {
            headers
                .get("host")
                .map(|s| s.as_str().unwrap_or(""))
                .unwrap_or("")
        })
        .unwrap_or(""))
}
fn get_path(settings: &serde_json::Value) -> Result<&str, anyhow::Error> {
    Ok(settings
        .get("path")
        .map(|path| path.as_str().unwrap_or("/"))
        .unwrap_or("/"))
}
fn get_ws_settings(
    stream_settings: &serde_json::Value,
) -> Result<&serde_json::Value, anyhow::Error> {
    stream_settings
        .get("wsSettings")
        .ok_or_else(|| anyhow!("No wsSettings"))
}
fn get_sni(settings: &serde_json::Value) -> Result<&str, anyhow::Error> {
    settings
        .get("serverName")
        .ok_or_else(|| anyhow!("No sni"))?
        .as_str()
        .ok_or_else(|| anyhow!("sni should be a String"))
}
fn get_tls_settings(
    stream_settings: &serde_json::Value,
) -> Result<&serde_json::Value, anyhow::Error> {
    stream_settings
        .get("tlsSettings")
        .ok_or_else(|| anyhow!("No tlsSettings"))
}
fn get_network(settings: &serde_json::Value) -> Result<&str, anyhow::Error> {
    Ok(
        match settings
            .get("network")
            .ok_or_else(|| anyhow!("No Network"))?
            .as_str()
            .ok_or_else(|| anyhow!("Network should be a String"))?
        {
            "tcp" => "original",
            "ws" => "ws",
            "http" => "h2",
            _ => return Err(anyhow!("No transport existed")),
        },
    )
}

fn get_stream_settings(outbound: &serde_json::Value) -> Result<&serde_json::Value, anyhow::Error> {
    outbound
        .get("streamSettings")
        .ok_or_else(|| anyhow!("No streamSettings"))
}

fn get_password(server: &serde_json::Value) -> Result<&str, anyhow::Error> {
    server
        .get("password")
        .ok_or_else(|| anyhow!("No Password"))?
        .as_str()
        .ok_or_else(|| anyhow!("Password should be a String"))
}

fn get_port(server: &serde_json::Value) -> Result<u64, anyhow::Error> {
    server
        .get("port")
        .ok_or_else(|| anyhow!("No Port"))?
        .as_u64()
        .ok_or_else(|| anyhow!("Port should be an integer"))
}

fn get_address(server: &serde_json::Value) -> Result<&str, anyhow::Error> {
    server
        .get("address")
        .ok_or_else(|| anyhow!("No Address"))?
        .as_str()
        .ok_or_else(|| anyhow!("Address should be a String"))
}

fn get_server<'a>(
    outbound: &'a serde_json::Value,
    trojan: &str,
) -> Result<&'a serde_json::Value, anyhow::Error> {
    outbound
        .get("settings")
        .ok_or_else(|| anyhow!("No trojan-go settings"))?
        .get(trojan)
        .ok_or_else(|| anyhow!("No trojan-go settings"))?
        .get("servers")
        .ok_or_else(|| anyhow!("No trojan-go servers"))?
        .as_array()
        .ok_or_else(|| anyhow!("Servers should be an array"))?
        .get(0)
        .ok_or_else(|| anyhow!("Servers array is empty"))
}

fn protocol_str(outbound: &serde_json::Value) -> Result<&str, anyhow::Error> {
    outbound
        .get("protocol")
        .ok_or_else(|| anyhow!("No protocol specified"))?
        .as_str()
        .ok_or_else(|| anyhow!("Protocol should be a string"))
}

impl CoreTool for TrojanGo {
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

    fn get_version(&self) -> &str {
        &self.version
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

#[cfg(test)]
mod tests {

    use std::{thread::sleep, time::Duration};

    use super::*;
    use anyhow::Result;
    #[test]
    fn test_core_version() -> Result<()> {
        let core = match TrojanGo::new("trojan-go") {
            Ok(c) => c,
            Err(e) => {
                println!("Core Error :{}", e);
                return Ok(());
            }
        };
        dbg!(core.name, core.version);
        Ok(())
    }
    #[test]
    fn test_core_run() -> Result<()> {
        let mut core = match TrojanGo::new("trojan-go") {
            Ok(c) => c,
            Err(e) => {
                println!("Core Error :{}", e);
                return Ok(());
            }
        };

        assert_eq!(false, core.is_running());
        core.set_config("--help".to_string())?;
        core.run()?;
        sleep(Duration::from_millis(500));
        assert_eq!(false, core.is_running());

        core.set_config(r#""#.to_string())?;
        core.run()?;
        assert_eq!(true, core.is_running());

        core.restart()?;
        assert_eq!(true, core.is_running());
        sleep(Duration::from_millis(500));

        Ok(())
    }
}
