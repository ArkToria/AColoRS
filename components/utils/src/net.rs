use std::{net::TcpListener, time::Duration};

use tokio::{net::TcpStream, time::Instant};

pub fn tcp_get_available_port(mut range: std::ops::Range<u16>) -> Option<u16> {
    range.find(|port| tcp_port_is_available(*port))
}

pub fn tcp_port_is_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

pub async fn get_http_content<T>(url: T, proxy: &str) -> anyhow::Result<String>
where
    T: reqwest::IntoUrl,
{
    let mut client_builder = reqwest::Client::builder();
    if !proxy.is_empty() {
        client_builder = client_builder.proxy(reqwest::Proxy::all(proxy)?);
    }
    let client = client_builder.build()?;
    let response = client.get(url).send().await?;
    let result = response.text().await?;

    Ok(result)
}
pub async fn tcping(address: &str, timeout: Duration) -> tokio::io::Result<Duration> {
    let start = Instant::now();
    tokio::time::timeout(timeout, TcpStream::connect(address)).await??;
    Ok(Instant::now().duration_since(start))
}
#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_get() -> anyhow::Result<()> {
        let content = get_http_content("http://example.com/", "").await;
        assert!(content.is_ok());
        println!("{:?}", content);
        Ok(())
    }

    async fn test_ping(address: &'static str, count: i32) -> Option<tokio::io::Result<Duration>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(20000);

        (0..count).for_each(|i| {
            println!("Sender {}:", i);
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(tcping(address, Duration::from_secs(2)).await).await;
                println!("Sender {} Sent", i);
            });
        });
        rx.recv().await
    }

    #[tokio::test]
    async fn tcping_domain_test() -> anyhow::Result<()> {
        let latency = test_ping("example.com:443", 10).await.unwrap();

        println!("{:?}", latency);
        assert!(latency.is_ok());
        Ok(())
    }
    #[tokio::test]
    async fn tcping_ip_test() -> anyhow::Result<()> {
        let latency = test_ping("93.184.216.34:443", 10).await.unwrap();
        println!("{:?}", latency);
        assert!(latency.is_ok());
        Ok(())
    }
}
