use std::process::ChildStdout;

use anyhow::Result;
pub trait CoreTool {
    fn run(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn get_is_running(&mut self) -> bool;
    fn set_config(&mut self, config: String) -> Result<()>;
    fn set_config_by_node_and_socks_inbound(
        &mut self,
        node_data: &core_data::NodeData,
        inbound: &config_manager::SOCKS5Inbound,
    ) -> Result<()>;

    fn restart(&mut self) -> Result<()> {
        if self.get_is_running() {
            self.stop()?;
        }
        self.run()
    }
    fn update_config(&mut self, config: String) -> Result<()> {
        self.set_config(config)?;
        if self.get_is_running() {
            self.restart()?;
        }
        Ok(())
    }

    fn get_stdout(&mut self) -> Option<ChildStdout>;
    fn get_version(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_config(&self) -> &str;
}
