use std::{
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use spdlog::error;

pub fn get_current_time() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(t) => t.as_secs(),
        Err(e) => {
            error!("SystemTime before UNIX EPOCH: {}", e);
            process::exit(1);
        }
    }
}
