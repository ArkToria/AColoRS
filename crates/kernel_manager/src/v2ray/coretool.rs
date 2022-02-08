use std::process::Child;

use anyhow::Result;

use crate::core::CoreTool;

struct V2rayCore {
    config: String,
    child_process: Option<Child>,
}

impl CoreTool<String> for V2rayCore {
    fn run(&mut self) -> Result<()> {
        todo!()
    }

    fn stop(&mut self) -> Result<()> {
        todo!()
    }

    fn is_running(&self) -> bool {
        !self.child_process.is_none()
    }

    fn set_config(&mut self, config: String) -> Result<()> {
        self.config = config;
        Ok(())
    }
}
