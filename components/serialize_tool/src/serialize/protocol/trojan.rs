use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::serialize::serializer::check_is_default_and_delete;
use crate::serialize::serializetool::URLMetaObject;
use core_data::NodeData;
use core_protobuf::acolors_proto::EntryType;
use core_protobuf::v2ray_proto::*;

pub fn trojan_outbound_from_url(url_str: String) -> Result<NodeData> {
    let meta = trojan_decode(&url_str)?;
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

    let name = urlencoding::decode(&meta.name)?.to_string();

    node.protocol = EntryType::Trojan.into();
    node.name = name;
    node.address = server.address.clone();
    node.port = server.port as i32;
    node.password = server.password.clone();
    node.raw = serde_json::to_string_pretty(&raw)?;
    node.url = url_str;

    Ok(node)
}

fn trojan_decode(url_str: &str) -> Result<URLMetaObject> {
    // url scheme:
    // trojan://<password>@<host>:<port>?sni=<server_name>&allowinsecure=<allow_insecure>&alpn=h2%0Ahttp/1.1#<name>
    let re = regex::Regex::new(r#"(\w+)://([^/@:]*)@([^@:]*):([^:]*)\?([^%]*)%0A([^#]*)#([^#]*)"#)?;
    let caps = match re.captures(url_str) {
        Some(c) => c,
        None => {
            return Err(anyhow!("Failed to parse sip002 url"));
        }
    };

    let mut meta = URLMetaObject {
        name: caps[7].to_string(),
        ..Default::default()
    };

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

    let query: Vec<&str> = caps[5].split('&').into_iter().collect();
    let mut map = HashMap::new();
    for pair in &query {
        let pair: Vec<&str> = pair.split('=').collect();
        if pair.len() != 2 {
            return Err(anyhow!("Wrong query arguments"));
        }
        let (key, value) = (pair[0], pair[1]);
        map.insert(key.to_string(), value.to_string());
    }

    let mut stream = StreamSettingsObject::default();
    let mut tls = stream_settings_object::TlsObject::default();

    if map.contains_key("sni") {
        tls.server_name = map.remove("sni").unwrap_or_else(|| "".to_string());
    }

    if map.contains_key("allowinsecure") {
        let allowinsecure: bool = map
            .remove("allowinsecure")
            .unwrap_or_else(|| "false".to_string())
            .parse()?;
        tls.allow_insecure = allowinsecure;
    } else {
        stream.network = "tcp".to_string();
        stream.security = "tls".to_string();
    }

    if map.contains_key("alpn") {
        let alpn = map.remove("alpn").unwrap_or_else(|| "".to_string());

        let mut values: Vec<&str> = Vec::new();
        if alpn.contains(',') {
            values = alpn.split(',').collect();
        } else if alpn.contains('\n') {
            values = alpn.split('\n').collect();
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
