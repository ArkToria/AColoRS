use anyhow::Result;
pub trait CoreTool<ConfigType> {
    fn run(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
    fn set_config(&mut self, config: ConfigType) -> Result<()>;
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
}
