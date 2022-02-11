use std::{
    ffi::OsString,
    io::{Read, Write},
    process::{Child, Command, Stdio},
};

use anyhow::{anyhow, Result};

use crate::core::CoreTool;

#[derive(Debug)]
pub struct V2rayCore {
    config: String,
    child_process: Option<Child>,
    path: OsString,
    name: String,
    version: semver::Version,
}

impl V2rayCore {
    fn spawn_version_process(path: &OsString) -> Result<Child> {
        let mut process = Command::new(path)
            .arg("version")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let mut stdin = match process.stdin.take() {
            Some(cs) => cs,
            None => return Err(anyhow!("No ChildStdin")),
        };
        stdin.write_all(b"}{")?;
        Ok(process)
    }

    fn get_name_and_version(child: Child) -> Result<(String, String)> {
        let mut stdout = match child.stdout {
            Some(out) => out,
            None => return Err(anyhow!("No child stdout")),
        };
        let mut buf = [0; 20];
        stdout.read_exact(&mut buf)?;

        let core_info = String::from_utf8_lossy(&buf);
        let mut info_split = core_info.split(' ');
        let name = info_split.next().unwrap_or("").to_string();
        let version = info_split.next().unwrap_or("").to_string();
        Ok((name, version))
    }
}

impl CoreTool<String> for V2rayCore {
    fn new(path: OsString) -> Result<Self> {
        let output = Self::spawn_version_process(&path)?;

        let (name, version) = Self::get_name_and_version(output)?;

        let semver_version = semver::Version::parse(&version)?;

        Ok(Self {
            name,
            version: semver_version,
            path,
            config: String::new(),
            child_process: None,
        })
    }

    fn run(&mut self) -> Result<()> {
        let mut child = Command::new(&self.path)
            .arg("--config=stdin:")
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
        match self.child_process.as_mut() {
            Some(child) => child.stdout.take(),
            None => None,
        }
    }

    fn get_version(&self) -> semver::Version {
        self.version.clone()
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {

    use std::{thread::sleep, time::Duration};

    use super::*;
    use anyhow::Result;
    #[test]
    fn test_core_version() -> Result<()> {
        let core = V2rayCore::new("v2ray".into())?;
        dbg!(core.name, core.version);
        Ok(())
    }
    #[test]
    fn test_core_run() -> Result<()> {
        let mut core = V2rayCore::new("v2ray".into())?;

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
