use std::{
    ffi::OsString,
    io::Write,
    process::{Child, Command, Stdio},
};

use anyhow::{anyhow, Result};

use crate::core::CoreTool;

#[derive(Debug)]
pub struct V2rayCore {
    config: String,
    child_process: Option<Child>,
    path: OsString,
}

impl CoreTool<String> for V2rayCore {
    fn new(path: OsString) -> Self {
        Self {
            path,
            config: String::new(),
            child_process: None,
        }
    }

    fn run(&mut self) -> Result<()> {
        let mut child = Command::new(&self.path)
            .arg("--config=stdin:")
            .stdin(Stdio::piped())
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
}

#[cfg(test)]
mod tests {

    use std::{process::ExitStatus, thread::sleep, time::Duration};

    use super::*;
    use anyhow::Result;
    impl V2rayCore {
        pub fn wait(&mut self) -> Result<ExitStatus> {
            let child = match self.child_process.as_mut() {
                Some(c) => c,
                None => return Err(anyhow!("child not exists")),
            };

            Ok(child.wait()?)
        }
    }
    #[test]
    fn test_core_run() -> Result<()> {
        let mut core = V2rayCore::new("v2ray".into());

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

        core.wait()?;

        Ok(())
    }
}
