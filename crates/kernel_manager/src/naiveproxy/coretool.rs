use std::{
    ffi::{OsStr, OsString},
    io::{Read, Write},
    process::{Child, Command, Stdio},
};

use anyhow::{anyhow, Result};
use spdlog::warn;

use crate::core::CoreTool;

#[derive(Debug)]
pub struct NaiveProxy {
    config: String,
    child_process: Option<Child>,
    path: OsString,
    name: String,
    version: String,
}

impl NaiveProxy {
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
            warn!("NaiveProxy currently don't have http inbounds.");
        }
        Self::set_inbounds(inbounds, &mut config)?;

        let protocol = node_data.url.split("://").next().unwrap_or("");
        match protocol {
            "naive+https" | "naive+quic" => {
                config.push_str(&format!(
                    " --proxy={}",
                    &node_data.url[6..node_data.url.len()]
                ));
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
            " --listen=socks://{}:{}",
            socks.listen, socks.port
        ));
        Ok(())
    }
}

impl CoreTool for NaiveProxy {
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
        let core = match NaiveProxy::new("naiveproxy") {
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
        let mut core = match NaiveProxy::new("naiveproxy") {
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

        core.set_config(
            "--listen=socks://127.0.0.1:4444 --proxy=https://username:password@127.0.0.1:443"
                .to_string(),
        )?;

        core.run()?;
        assert_eq!(true, core.is_running());

        core.restart()?;
        assert_eq!(true, core.is_running());
        sleep(Duration::from_millis(500));

        Ok(())
    }
}
