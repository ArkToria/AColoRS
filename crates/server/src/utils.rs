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

    #[tokio::test]
    async fn test_get_and_decode() -> anyhow::Result<()> {
        let content =
            get_http_content("https://hk.khonoka.com/subnode/getproxyyuruiboom/proxy").await?;
        let decoded_content = base64::decode(content).unwrap_or(Vec::new());
        let res = String::from_utf8(decoded_content)?;
        println!("{}", res);
        Ok(())
    }
}
