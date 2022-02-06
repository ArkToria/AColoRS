use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::protobuf::acolors_proto::EntryType;
use crate::protobuf::v2ray_proto::*;
use crate::serialize::serialize::URLMetaObject;
use crate::serialize::serializer::check_is_default_and_delete;
use crate::NodeData;

pub fn vmess_outbound_from_base64(url_str: String) -> Result<NodeData> {
    let meta = vmess_base64_decode(&url_str)?;
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

    let name = urlencoding::decode(&meta.name)?.to_string();

    node.protocol = EntryType::Vmess.into();
    node.name = name;
    node.address = server.address.clone();
    node.port = server.port as i32;
    node.password = user.id.clone();
    node.raw = serde_json::to_string_pretty(&raw)?;
    node.url = url_str.clone();

    Ok(node)
}

fn vmess_base64_decode(url_str: &str) -> Result<URLMetaObject> {
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
