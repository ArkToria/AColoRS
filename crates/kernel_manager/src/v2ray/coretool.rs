use std::{
    ffi::{OsStr, OsString},
    io::{Read, Write},
    process::{Child, Command, Stdio},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use serialize_tool::serialize::serializer::check_is_default_and_delete;

use crate::core::CoreTool;

use super::configtool::set_inbound_object;
use core_protobuf::v2ray_proto::{OutboundObject, V2RayConfig};

#[derive(Debug)]
pub struct V2RayCore {
    config: String,
    child_process: Option<Child>,
    path: OsString,
    name: String,
    version: String,
}

impl V2RayCore {
    pub fn new<S: AsRef<OsStr> + ?Sized>(path: &S) -> Result<Self> {
        let output = Self::spawn_version_process(path.as_ref())?;

        let (name, version) = Self::get_name_and_version(output)?;

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
            .arg("version")
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
        let mut node_config = V2RayConfig::default();
        let mut json;

        set_inbound_object(&mut node_config, inbounds);

        if !node_data.url.contains("://") {
            json = config_to_json(&node_config, &node_data.raw)?;
        } else {
            let mut outbound = json_to_outbound(&node_data.raw)?;

            if outbound.tag.is_empty() {
                outbound.tag = "PROXY".to_string();
            }

            node_config.outbounds.push(outbound);

            json = config_to_json(&node_config, "")?;
        }

        check_is_default_and_delete(&mut json);

        Ok(json.to_string())
    }
}

impl CoreTool for V2RayCore {
    fn run(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(anyhow!("Core is running"));
        }

        let mut child = Command::new(&self.path)
            .args(&["run", "-format", "json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut stdin = match child.stdin.take() {
            Some(cs) => cs,
            None => return Err(anyhow!("No ChildStdin")),
        };

        stdin.write_all(self.config.as_bytes())?;

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

fn json_to_outbound(json_str: &str) -> Result<OutboundObject, serde_json::Error> {
    serde_json::from_str(json_str)
}

fn config_to_json(origin_config: &V2RayConfig, outbound_raw: &str) -> Result<serde_json::Value> {
    let mut root = serde_json::to_value(&origin_config)?;

    if root["inbounds"].is_null() {
        return Ok(serde_json::Value::Null);
    };

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
        fix_format(&mut root, vec!["inbounds", "outbounds"]);
    } else {
        fix_format(&mut root, vec!["inbounds"]);

        let outbound = serde_json::Value::from_str(outbound_raw)?;

        if !outbound.is_null() {
            root["outbounds"] = serde_json::Value::Array(vec![outbound]);
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
        let core = V2RayCore::new("v2ray")?;
        dbg!(core.name, core.version);
        Ok(())
    }
    #[test]
    fn test_core_run() -> Result<()> {
        let mut core = V2RayCore::new("v2ray")?;

        assert_eq!(false, core.is_running());
        core.set_config("}{".to_string())?;
        core.run()?;
        sleep(Duration::from_millis(500));
        assert_eq!(false, core.is_running());

        core.set_config("{}".to_string())?;

        core.run()?;
        assert_eq!(true, core.is_running());

        core.restart()?;
        assert_eq!(true, core.is_running());
        sleep(Duration::from_millis(500));

        Ok(())
    }
}
