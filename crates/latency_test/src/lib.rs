use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::net::TcpStream;

pub async fn tcping(address: &str, timeout: Duration) -> Result<Duration> {
    let start = Instant::now();
    tokio::time::timeout(timeout, TcpStream::connect(address)).await??;
    Ok(Instant::now().duration_since(start))
}
