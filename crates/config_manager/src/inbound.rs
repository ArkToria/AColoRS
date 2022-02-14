use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Inbounds {
    #[serde(default)]
    pub inbounds: Vec<Inbound>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Inbound {
    Http(HTTPInbound),
    Socks5(SOCKS5Inbound),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HTTPInbound {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub listen: String,
    #[serde(default)]
    pub port: u32,
    #[serde(default)]
    pub allow_transparent: bool,
    #[serde(default)]
    pub timeout: i64,
    #[serde(default)]
    pub user_level: i32,
    #[serde(default)]
    pub auth: Option<Auth>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SOCKS5Inbound {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub listen: String,
    #[serde(default)]
    pub port: u32,
    #[serde(default)]
    pub udp_enable: bool,
    #[serde(default)]
    pub udp_ip: String,
    #[serde(default)]
    pub user_level: i32,
    #[serde(default)]
    pub auth: Option<Auth>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

impl Inbounds {
    pub fn from_str(input: &str) -> Result<Inbounds, serde_json::Error> {
        serde_json::from_str(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_inbounds_deserialize() -> anyhow::Result<()> {
        let test_empty = Inbounds::from_str("{}")?;
        dbg!(&test_empty);
        assert_eq!(true, test_empty.inbounds.is_empty());
        Ok(())
    }
}
