mod signal;
pub use signal::*;

use spdlog::error;
pub fn send_or_error_print<T>(sender: &tokio::sync::broadcast::Sender<T>, content: T) {
    if let Err(e) = sender.send(content) {
        error!("Consumer Reply Error: {}", e);
    }
}
