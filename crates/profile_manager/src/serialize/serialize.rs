use anyhow::{anyhow, Result};

use crate::protobuf::v2ray_proto::*;
use crate::NodeData;

use super::protocol::shadowsocks::shadowsocks_outbound_from_url;
use super::protocol::trojan::trojan_outbound_from_url;
use super::protocol::vmess::vmess_outbound_from_base64;
use spdlog::error;

#[derive(Default)]
pub struct URLMetaObject {
    pub name: String,
    pub outbound: OutboundObject,
}

pub fn get_nodes_from_base64(base64: &str) -> anyhow::Result<Vec<NodeData>> {
    let mut nodes = Vec::new();
    let url_str = String::from_utf8(base64::decode(base64)?)?;
    let url_lines = url_str.lines();

    url_lines.into_iter().for_each(|node_url| {
        let node = decode_outbound_from_url(node_url);
        match node {
            Ok(n) => nodes.push(n),
            Err(e) => {
                error!("Node url parse error : {}", e);
            }
        }
    });

    Ok(nodes)
}

pub fn decode_outbound_from_url<T: Into<String>>(url: T) -> Result<NodeData> {
    let url_string: String = url.into();
    let scheme = url_string.split("://").next().unwrap_or("");
    if scheme.is_empty() {
        return Err(anyhow!("No scheme"));
    }
    match scheme {
        "vmess" => return vmess_outbound_from_base64(url_string),
        "trojan" => return trojan_outbound_from_url(url_string),
        "ss" => return shadowsocks_outbound_from_url(url_string),
        _ => return Err(anyhow!("Not implemented")),
    };
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use anyhow::Result;
    use regex::Regex;

    #[test]
    fn test_vmess() -> Result<()> {
        let t=decode_outbound_from_url("vmess://eyJhZGQiOiJ0ZXN0MiIsImFpZCI6MzEyLCJob3N0IjoiZmQiLCJpZCI6InRlc3QzIiwibmV0Ijoid3MiLCJwYXRoIjoiYWZkIiwicG9ydCI6MTQyLCJwcyI6InRlc3QxIiwic2N5IjoiY2hhY2hhMjAtcG9seTEzMDUiLCJzbmkiOiI0MTIiLCJ0bHMiOiJ0bHMiLCJ0eXBlIjoibm9uZSIsInYiOiIyIn0=@");
        let data = match t {
            Ok(d) => d,
            Err(e) => {
                println!("Error: {}", e);
                return Ok(());
            }
        };
        println!("{}", data.raw);
        assert_eq!(
            data.raw,
            r#"{
  "protocol": "vmess",
  "sendThrough": "0.0.0.0",
  "settings": {
    "vmess": {
      "vnext": [
        {
          "address": "test2",
          "port": 142,
          "users": [
            {
              "alterId": 312,
              "id": "test3",
              "security": "chacha20-poly1305"
            }
          ]
        }
      ]
    }
  },
  "streamSettings": {
    "network": "ws",
    "security": "tls",
    "tlsSettings": {
      "serverName": "412"
    },
    "wsSettings": {
      "headers": {
        "Host": "fd"
      },
      "path": "afd"
    }
  }
}"#
        );
        Ok(())
    }
    #[test]
    fn test_trojan() -> Result<()> {
        let data = decode_outbound_from_url(
            "trojan://password@host:756?sni=servername&allowinsecure=false&alpn=h2,http/1.1%0Ahttp/1.1#name",
        )?;
        println!("{:?}", data);
        println!("{}", data.raw);
        assert_eq!(
            r#"{
  "protocol": "trojan",
  "sendThrough": "0.0.0.0",
  "settings": {
    "trojan": {
      "servers": [
        {
          "address": "host",
          "password": "password",
          "port": 756
        }
      ]
    }
  },
  "streamSettings": {
    "tlsSettings": {
      "alpn": [
        "h2",
        "http/1.1"
      ],
      "serverName": "servername"
    }
  }
}"#,
            data.raw
        );
        Ok(())
    }
    #[test]
    fn test_ss() -> Result<()> {
        let data = decode_outbound_from_url("ss://YWVzLTI1Ni1nY206dGVzdDM=@test2:123#test1")?;
        println!("{:?}", data);
        println!("{}", data.raw);
        assert_eq!(
            r#"{
  "protocol": "shadowsocks",
  "sendThrough": "0.0.0.0",
  "settings": {
    "shadowsocks": {
      "servers": [
        {
          "address": "test2",
          "method": "aes-256-gcm",
          "password": "test3",
          "port": 123
        }
      ]
    }
  }
}"#,
            data.raw
        );
        Ok(())
    }
    #[test]
    fn test_regex() -> Result<()> {
        let re = Regex::new(r#"(\w+)://([^/@:]*)@([^@:]*):([^:#]*)#([^#]*)"#)?;
        let ss = "ss://YWVzLTI1Ni1nY206dGVzdDM=@test2:123#test1";
        let list = re.captures(ss).unwrap();
        println!("{:?}", list);

        let re = Regex::new(r#"(\w+)://([^/@:]*)@([^@:]*):([^:]*)\?([^%]*)%0A([^#]*)#([^#]*)"#)?;
        let trojan =
            "trojan://password@host:756?sni=servername&allowinsecure=false&alpn=h2%0Ahttp/1.1#name";
        let list = re.captures(trojan).unwrap();
        let query: Vec<&str> = list[5].split("&").into_iter().collect();
        let mut map = HashMap::new();
        for pair in &query {
            let pair: Vec<&str> = pair.split("=").collect();
            if pair.len() != 2 {
                return Err(anyhow!("Wrong query arguments"));
            }
            let (key, value) = (pair[0], pair[1]);
            map.insert(key.to_string(), value.to_string());
        }

        println!("{:?}", list);
        println!("{:?}", query);
        println!("{} {} {}", map["sni"], map["allowinsecure"], map["alpn"]);

        Ok(())
    }
}
