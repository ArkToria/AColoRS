pub mod convert;
mod cores;
mod inbound;

pub use cores::*;
pub use inbound::*;

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
