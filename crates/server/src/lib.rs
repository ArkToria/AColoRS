use errors::{Error as ACrossError, Result};
use log::debug;
pub fn serve(interface: &str, interface_port: u16) -> Result<()> {
    debug!(
        "ACross Server is starting at {}:{}",
        interface, interface_port
    );
    Ok(())
}
