use anyhow::Result;
trait CoreTools {
    fn run() -> Result<()>;
    fn stop() -> Result<()>;
    fn is_running() -> bool;
}
