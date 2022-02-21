mod signal;
pub use signal::*;
use spdlog::warn;

pub fn send_or_warn_print<T>(sender: &tokio::sync::broadcast::Sender<T>, content: T) {
    if let Err(e) = sender.send(content) {
        warn!("Consumer Reply Error: {}", e);
    }
}
