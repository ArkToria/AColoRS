use anyhow::{anyhow, Result};

use crate::serialize::serializer::check_is_default_and_delete;
use crate::serialize::serializetool::URLMetaObject;
use crate::NodeData;
use core_protobuf::acolors_proto::EntryType;
use core_protobuf::v2ray_proto::*;

pub fn shadowsocks_outbound_from_url(url_str: String) -> Result<NodeData> {
    let meta = sip002_decode(&url_str)?;
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

    let name = urlencoding::decode(&meta.name)?.to_string();

    node.protocol = EntryType::Vmess.into();
    node.name = name;
    node.address = server.address.clone();
    node.port = server.port as i32;
    node.password = server.password.clone();
    node.raw = serde_json::to_string_pretty(&raw)?;
    node.url = url_str;

    Ok(node)
}

fn sip002_decode(url_str: &str) -> Result<URLMetaObject> {
    // url scheme:
    // ss://<websafe-base64-encode-utf8(method:password)>@hostname:port/?plugin"#"tag

    let re = regex::Regex::new(r#"(\w+)://([^/@:]*)@([^@:]*):([^:#]*)#([^#]*)"#)?;
    let caps = match re.captures(url_str) {
        Some(c) => c,
        None => {
            return Err(anyhow!("Failed to parse sip002 url"));
        }
    };

    let mut meta = URLMetaObject {
        name: caps[5].to_string(),
        ..Default::default()
    };

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

    let mut user_info = user_info.split(':');

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
