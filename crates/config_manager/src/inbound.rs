use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Inbounds {
    pub inbounds: Vec<Inbound>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Inbound {
    Http(HTTPInbound),
    Socks5(SOCKS5Inbound),
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct HTTPInbound {
    pub enable: bool,
    pub listen: String,
    pub port: u32,
    pub allow_transparent: bool,
    pub timeout: i64,
    pub user_level: i32,
    pub auth: Option<Auth>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SOCKS5Inbound {
    pub enable: bool,
    pub listen: String,
    pub port: u32,
    pub udp_enable: bool,
    pub udp_ip: String,
    pub user_level: i32,
    pub auth: Option<Auth>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Auth {
    pub enable: bool,
    pub username: String,
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
