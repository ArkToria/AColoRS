use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CoreList {
    pub cores: Vec<Core>,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Core {
    pub tag: String,
    pub name: String,
    pub path: String,
}

impl std::str::FromStr for CoreList {
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
        let test_empty = CoreList::from_str("{}")?;
        dbg!(&test_empty);
        assert_eq!(true, test_empty.cores.is_empty());
        Ok(())
    }
    #[test]
    fn test_inbounds_deserialize() -> anyhow::Result<()> {
        let inbounds = CoreList::from_str(
            r#"
{
  "cores": [
    {
      "tag": "v2tag",
      "name": "v2ray",
      "path": "/usr/bin/v2ray"
    },
    {
      "tag": "sstag",
      "name": "shadowsocks",
      "path": "/usr/bin/sslocal-rust"
    },
    {
      "tag": "trojantag",
      "name": "trojan",
      "path": "/usr/bin/trojan"
    }
  ]
}
        "#,
        )?;
        dbg!(&inbounds);
        Ok(())
    }
}
