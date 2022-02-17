use std::net::TcpListener;

pub fn tcp_get_available_port(mut range: std::ops::Range<u16>) -> Option<u16> {
    range.find(|port| tcp_port_is_available(*port))
}

pub fn tcp_port_is_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

pub async fn get_http_content<T: reqwest::IntoUrl>(url: T) -> anyhow::Result<String> {
    let response = reqwest::get(url).await?;
    let result = response.text().await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get() -> anyhow::Result<()> {
        println!("{}", get_http_content("https://example.com/").await?);
        Ok(())
    }
}
