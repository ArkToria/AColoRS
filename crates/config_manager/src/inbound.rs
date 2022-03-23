use serde::{Deserialize, Serialize};

fn local_ip_string() -> String {
    "127.0.0.1".to_string()
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Inbounds {
    pub socks5: Option<SOCKS5Inbound>,
    pub http: Option<HTTPInbound>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SOCKS5Inbound {
    pub enable: bool,
    pub listen: String,
    pub port: u32,
    pub udp_enable: bool,
    #[serde(default = "local_ip_string")]
    pub udp_ip: String,
    pub user_level: i32,
    pub auth: Option<Auth>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Auth {
    pub enable: bool,
    pub username: String,
    pub password: String,
}

impl std::str::FromStr for Inbounds {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
impl std::str::FromStr for SOCKS5Inbound {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    #[test]
    fn test_empty_inbounds_deserialize() -> anyhow::Result<()> {
        let test_empty = Inbounds::from_str("{}")?;
        dbg!(&test_empty);
        assert_eq!(true, test_empty.socks5.is_none());
        assert_eq!(true, test_empty.http.is_none());
        Ok(())
    }
    #[test]
    fn test_inbounds_deserialize() -> anyhow::Result<()> {
        let inbounds = Inbounds::from_str(
            r#"
{
    "socks5": {
        "enable": true,
        "listen": "127.0.0.1",
        "port": 4444,
        "udp_enable": true,
        "auth": {}
    },
    "http": {
        "enable": true,
        "listen": "127.0.0.1",
        "port": 4445,
        "auth": {
            "enable": true,
            "username": "testusername",
            "password": "testpassword"
        }
    }
}
        "#,
        )?;
        dbg!(&inbounds);
        Ok(())
    }
}
