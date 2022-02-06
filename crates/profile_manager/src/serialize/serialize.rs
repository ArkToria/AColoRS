use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::protobuf::acolors_proto::*;
use crate::protobuf::v2ray_proto::*;
use crate::NodeData;

use super::serializer::check_is_default_and_delete;

#[derive(Default)]
struct URLMetaObject {
    pub name: String,
    pub outbound: OutboundObject,
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

fn shadowsocks_outbound_from_url(url_str: String) -> Result<NodeData> {
    let meta = sip002_decode(url_str)?;
    let mut node = NodeData::default();

    let outbound = meta.outbound;
    let outbound_settings = match &outbound.settings {
        Some(s) => s,
        None => return Err(anyhow!("No OutboundSettings")),
    };
    let shadowsocks = match &outbound_settings.kind {
        Some(s) => match s {
            outbound_object::outbound_settings::Kind::Shadowsocks(ss) => ss,
            _ => return Err(anyhow!("Protocol Error")),
        },
        None => return Err(anyhow!("No OutboundSettings Kind")),
    };

    let server = &shadowsocks.servers[0];

    let mut raw = serde_json::to_value(&outbound)?;
    check_is_default_and_delete(&mut raw);

    node.protocol = EntryType::Vmess.into();
    node.name = meta.name.clone();
    node.address = server.address.clone();
    node.port = server.port as i32;
    node.password = server.password.clone();
    node.raw = serde_json::to_string_pretty(&raw)?;

    Ok(node)
}

fn sip002_decode(url_str: String) -> Result<URLMetaObject> {
    // url scheme:
    // ss://<websafe-base64-encode-utf8(method:password)>@hostname:port/?plugin"#"tag

    let re = regex::Regex::new(r#"(\w+)://([^/@:]*)@([^@:]*):([^:#]*)#([^#]*)"#)?;
    let caps = match re.captures(&url_str) {
        Some(c) => c,
        None => {
            return Err(anyhow!("Failed to parse sip002 url"));
        }
    };

    let mut meta = URLMetaObject::default();
    meta.name = caps[5].to_string();

    let mut outbound = &mut meta.outbound;
    outbound.protocol = "shadowsocks".into();
    outbound.send_through = "0.0.0.0".into();

    let mut outbound_settings = outbound_object::OutboundSettings::default();
    let mut shadowsocks = shadowsocks_object::OutboundConfigurationObject::default();
    let mut server = shadowsocks_object::ServerObject::default();

    let user_info = String::from_utf8(base64::decode(&caps[2])?)?;
    if user_info.is_empty() {
        return Err(anyhow!("Empty User Info"));
    }

    let mut user_info = user_info.split(":");

    server.address = caps[3].to_string();
    server.port = caps[4].parse()?;
    server.password = user_info.clone().last().unwrap_or("").to_string();
    server.method = user_info.next().unwrap_or("").to_string();

    shadowsocks.servers.push(server);
    outbound_settings.kind = Some(outbound_object::outbound_settings::Kind::Shadowsocks(
        shadowsocks,
    ));
    outbound.settings = Some(outbound_settings);
    Ok(meta)
}

fn trojan_outbound_from_url(url_str: String) -> Result<NodeData> {
    let meta = trojan_decode(url_str)?;
    let mut node = NodeData::default();

    let outbound = meta.outbound;
    let outbound_settings = match &outbound.settings {
        Some(s) => s,
        None => return Err(anyhow!("No OutboundSettings")),
    };
    let trojan = match &outbound_settings.kind {
        Some(s) => match s {
            outbound_object::outbound_settings::Kind::Trojan(trojan) => trojan,
            _ => return Err(anyhow!("Protocol Error")),
        },
        None => return Err(anyhow!("No OutboundSettings Kind")),
    };

    let server = &trojan.servers[0];

    let mut raw = serde_json::to_value(&outbound)?;
    check_is_default_and_delete(&mut raw);

    node.protocol = EntryType::Trojan.into();
    node.name = meta.name.clone();
    node.address = server.address.clone();
    node.port = server.port as i32;
    node.password = server.password.clone();
    node.raw = serde_json::to_string_pretty(&raw)?;

    Ok(node)
}

fn trojan_decode(url_str: String) -> Result<URLMetaObject> {
    // url scheme:
    // trojan://<password>@<host>:<port>?sni=<server_name>&allowinsecure=<allow_insecure>&alpn=h2%0Ahttp/1.1#<name>
    let re = regex::Regex::new(r#"(\w+)://([^/@:]*)@([^@:]*):([^:]*)\?([^%]*)%0A([^#]*)#([^#]*)"#)?;
    let caps = match re.captures(&url_str) {
        Some(c) => c,
        None => {
            return Err(anyhow!("Failed to parse sip002 url"));
        }
    };

    let mut meta = URLMetaObject::default();
    meta.name = caps[7].to_string();

    let mut outbound = &mut meta.outbound;
    outbound.protocol = "trojan".into();
    outbound.send_through = "0.0.0.0".into();

    let mut outbound_settings = outbound_object::OutboundSettings::default();
    let mut trojan = trojan_object::OutboundConfigurationObject::default();
    let mut server = trojan_object::ServerObject::default();

    if caps.len() < 7 {
        return Err(anyhow!("Parse trojan url error"));
    }

    server.address = caps[3].to_string();
    server.port = caps[4].parse()?;
    server.password = caps[2].to_string();

    let query: Vec<&str> = caps[5].split("&").into_iter().collect();
    let mut map = HashMap::new();
    for pair in &query {
        let pair: Vec<&str> = pair.split("=").collect();
        if pair.len() != 2 {
            return Err(anyhow!("Wrong query arguments"));
        }
        let (key, value) = (pair[0], pair[1]);
        map.insert(key.to_string(), value.to_string());
    }

    let mut stream = StreamSettingsObject::default();
    let mut tls = stream_settings_object::TlsObject::default();

    if map.contains_key("sni") {
        tls.server_name = map.remove("sni").unwrap_or("".to_string());
    }

    if map.contains_key("allowinsecure") {
        let allowinsecure: bool = map
            .remove("allowinsecure")
            .unwrap_or("false".to_string())
            .parse()?;
        tls.allow_insecure = allowinsecure;
    } else {
        stream.network = "tcp".to_string();
        stream.security = "tls".to_string();
    }

    if map.contains_key("alpn") {
        let alpn = map.remove("alpn").unwrap_or("".to_string());

        let mut values: Vec<&str> = Vec::new();
        if alpn.contains(',') {
            values = alpn.split(',').collect();
        } else if alpn.contains('\n') {
            values = alpn.split(',').collect();
        }

        if values.is_empty() {
            tls.alpn.push(alpn);
        } else {
            values.into_iter().for_each(|s| {
                tls.alpn.push(s.to_string());
            });
        }
    } else {
        tls.alpn.push("http/1.1".to_string());
    }

    stream.tls_settings = Some(tls);
    outbound.stream_settings = Some(stream);
    trojan.servers.push(server);
    outbound_settings.kind = Some(outbound_object::outbound_settings::Kind::Trojan(trojan));
    outbound.settings = Some(outbound_settings);

    Ok(meta)
}

fn vmess_outbound_from_base64(url_str: String) -> Result<NodeData> {
    let meta = vmess_base64_decode(url_str)?;
    let mut node = NodeData::default();

    let outbound = meta.outbound;
    let outbound_settings = match &outbound.settings {
        Some(s) => s,
        None => return Err(anyhow!("No OutboundSettings")),
    };
    let vmess = match &outbound_settings.kind {
        Some(s) => match s {
            outbound_object::outbound_settings::Kind::Vmess(vm) => vm,
            _ => return Err(anyhow!("Protocol Error")),
        },
        None => return Err(anyhow!("No OutboundSettings Kind")),
    };

    let server = &vmess.vnext[0];
    let user = &server.users[0];

    let mut raw = serde_json::to_value(&outbound)?;
    check_is_default_and_delete(&mut raw);

    node.protocol = EntryType::Vmess.into();
    node.name = meta.name.clone();
    node.address = server.address.clone();
    node.port = server.port as i32;
    node.password = user.id.clone();
    node.raw = serde_json::to_string_pretty(&raw)?;

    Ok(node)
}

fn vmess_base64_decode(url_str: String) -> Result<URLMetaObject> {
    // url scheme:
    // vmess://<base64EncodeJson>
    // {
    //     "v": "2",
    //     "ps": "Names",
    //     "add": "111.111.111.111",
    //     "port": "32000",
    //     "id": "1386f85e-657b-4d6e-9d56-78badb75e1fd",
    //     "aid": "100",
    //     "scy": "zero",
    //     "net": "tcp",
    //     "type": "none",
    //     "host": "www.bbb.com",
    //     "path": "/",
    //     "tls": "tls",
    //     "sni": "www.ccc.com"
    //  }
    let mut info = url_str.split("://").last().unwrap_or("");
    if info.ends_with('@') {
        info = &info[0..info.len() - 2];
    }
    if info.is_empty() {
        return Err(anyhow!("No Content"));
    };

    let decode_vec = base64::decode(info)?;
    let base64_str = String::from_utf8(decode_vec)?;

    let root: Value = serde_json::from_str(&base64_str)?;

    let mut meta = URLMetaObject::default();

    if let Value::String(s) = &root["ps"] {
        meta.name = s.clone();
    };

    let outbound = &mut meta.outbound;
    outbound.protocol = "vmess".into();
    outbound.send_through = "0.0.0.0".into();

    let mut outbound_settings = outbound_object::OutboundSettings::default();
    let mut vmess = vmess_object::OutboundConfigurationObject::default();
    let mut server = vmess_object::ServerObject::default();
    let mut user = vmess_object::UserObject::default();
    let mut stream = StreamSettingsObject::default();

    if (!root["add"].is_null()) && (!root["port"].is_null()) {
        server.address = root["add"].as_str().unwrap_or("default").to_string();

        match &root["port"] {
            Value::Number(n) => server.port = n.as_u64().unwrap_or(0) as u32,
            Value::String(s) => server.port = s.parse().unwrap_or(0),
            _ => return Err(anyhow!("Port illegal")),
        }
    } else {
        return Err(anyhow!("No address|port"));
    }

    match &root["id"] {
        Value::String(s) => user.id = s.clone(),
        _ => return Err(anyhow!("No id")),
    }

    user.alter_id = 0;
    match &root["aid"] {
        Value::Number(n) => user.alter_id = n.as_u64().unwrap_or(0) as i32,
        Value::String(s) => user.alter_id = s.parse().unwrap_or(0),
        _ => {}
    }

    user.security = "auto".into();
    if let Value::String(s) = &root["scy"] {
        user.security = s.clone();
    }

    if let Value::String(network) = &root["net"] {
        if network == "h2" {
            stream.network = "http".into();
        } else {
            stream.network = network.clone();
        }

        match stream.network.as_str() {
            "tcp" => {
                // TODO: TCP support
                //                auto tcp = stream->mutable_tcpsettings();

                //                if (root.contains("type")) {
                //                    auto header = tcp->mutable_header();
                //                    header->set_type(root["type"].get<std::string>());
                //                }
            }
            "http" => {
                let mut http2 = transport_object::HttpObject::default();

                if let Value::String(host) = &root["host"] {
                    let content = host.trim().split(',');
                    for host in content {
                        if !host.is_empty() {
                            http2.host.push(host.trim().to_string());
                        }
                    }
                }

                if let Value::String(path) = &root["path"] {
                    http2.path = path.clone();
                }

                stream.http_settings = Some(http2);
            }
            "ws" => {
                let mut websocket = transport_object::WebSocketObject::default();

                if let Value::String(host) = &root["host"] {
                    let headers = &mut websocket.headers;
                    headers.insert("Host".into(), host.clone());
                }

                if let Value::String(path) = &root["path"] {
                    websocket.path = path.clone();
                }

                stream.ws_settings = Some(websocket);
            }
            "grpc" => {
                let mut grpc = transport_object::GrpcObject::default();

                if let Value::String(path) = &root["path"] {
                    grpc.service_name = path.clone();
                }

                stream.grpc_settings = Some(grpc);
            }
            "quic" => {
                let mut quic = transport_object::QuicObject::default();

                if let Value::String(quic_type) = &root["type"] {
                    let mut header = transport_object::quic_object::HeaderObject::default();
                    header.r#type = quic_type.clone();

                    quic.header = Some(header);
                }

                if let Value::String(host) = &root["host"] {
                    quic.security = host.clone();
                }

                if let Value::String(path) = &root["path"] {
                    quic.key = path.clone();
                }

                stream.quic_settings = Some(quic);
            }

            _ => {}
        }
    }

    if let Value::String(tls) = &root["tls"] {
        stream.security = tls.clone();
    }

    if let Value::String(sni) = &root["sni"] {
        let mut tls = stream_settings_object::TlsObject::default();
        tls.server_name = sni.clone();
        stream.tls_settings = Some(tls);
    }

    outbound.stream_settings = Some(stream);
    server.users.push(user);
    vmess.vnext.push(server);
    outbound_settings.kind = Some(outbound_object::outbound_settings::Kind::Vmess(vmess));
    outbound.settings = Some(outbound_settings);
    Ok(meta)
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
