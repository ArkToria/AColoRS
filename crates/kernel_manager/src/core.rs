use std::{ffi::OsStr, process::ChildStdout};

use anyhow::Result;
pub trait CoreTool<ConfigType> {
    fn new<S: AsRef<OsStr> + ?Sized>(path: &S) -> Result<Self>
    where
        Self: Sized;
    fn run(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn is_running(&mut self) -> bool;
    fn set_config(&mut self, config: ConfigType) -> Result<()>;

    fn generate_config(
        node_data: &core_data::NodeData,
        inbounds: &config_manager::Inbounds,
    ) -> Result<ConfigType>;

    fn restart(&mut self) -> Result<()> {
        if self.is_running() {
            self.stop()?;
        }
        self.run()
    }
    fn update_config(&mut self, config: ConfigType) -> Result<()> {
        self.set_config(config)?;
        if self.is_running() {
            self.restart()?;
        }
        Ok(())
    }

    fn get_stdout(&mut self) -> Option<ChildStdout>;
    fn get_version(&self) -> semver::Version;
    fn get_name(&self) -> &str;
    fn get_config(&self) -> &str;
}
