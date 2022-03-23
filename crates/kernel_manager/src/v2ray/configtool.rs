use std::str::FromStr;

use anyhow::Result;
use core_protobuf::v2ray_proto::*;
use core_protobuf::v2ray_proto::{
    inbound_object::{inbound_settings, InboundSettings},
    InboundObject, V2RayConfig,
};
use serialize_tool::serialize::serializer::check_is_default_and_delete;

pub fn set_inbound_object(config: &mut V2RayConfig, inbounds: &config_manager::Inbounds) {
    set_inbound_http_object(&inbounds.http.as_ref(), config);
    set_inbound_socks5_object(&inbounds.socks5.as_ref(), config);
}

pub fn set_inbound_http_object(
    inbounds: &Option<&config_manager::HTTPInbound>,
    config: &mut V2RayConfig,
) {
    let http_settings = inbounds;
    if let Some(http_inbound) = http_settings {
        if http_inbound.enable {
            let mut http_inbound_configuration_object = http_object::InboundConfigurationObject {
                allow_transparent: http_inbound.allow_transparent,
                timeout: http_inbound.timeout as u32,
                user_level: http_inbound.user_level,
                ..Default::default()
            };

            let auth_setting = &http_inbound.auth;
            if let Some(auth) = auth_setting {
                if auth.enable {
                    let auth = core_protobuf::v2ray_proto::http_object::AccountObject {
                        user: auth.username.clone(),
                        pass: auth.password.clone(),
                    };
                    http_inbound_configuration_object.accounts.push(auth);
                }
            }

            let http = InboundSettings {
                kind: Some(inbound_settings::Kind::Http(
                    http_inbound_configuration_object,
                )),
            };

            let inbound = InboundObject {
                listen: http_inbound.listen.clone(),
                port: http_inbound.port,
                protocol: "http".to_string(),
                tag: "HTTP_IN".to_string(),
                settings: Some(http),
                ..Default::default()
            };
            config.inbounds.push(inbound);
        }
    }
}

pub fn set_inbound_socks5_object(
    inbounds: &Option<&config_manager::SOCKS5Inbound>,
    config: &mut V2RayConfig,
) {
    let socks5_settings = inbounds;
    if let Some(socks5_inbound) = socks5_settings {
        if socks5_inbound.enable {
            let mut socks5_inbound_configuration_object =
                socks_object::InboundConfigurationObject {
                    user_level: socks5_inbound.user_level,
                    ..Default::default()
                };

            if socks5_inbound.udp_enable {
                socks5_inbound_configuration_object.udp = true;
                socks5_inbound_configuration_object.ip = socks5_inbound.udp_ip.clone();
            }

            let auth_setting = &socks5_inbound.auth;
            if let Some(auth) = auth_setting {
                if auth.enable {
                    let auth = core_protobuf::v2ray_proto::socks_object::AccountObject {
                        user: auth.username.clone(),
                        pass: auth.password.clone(),
                    };
                    socks5_inbound_configuration_object.accounts.push(auth);
                }
            }

            let socks5 = InboundSettings {
                kind: Some(inbound_settings::Kind::Socks(
                    socks5_inbound_configuration_object,
                )),
            };

            let inbound = InboundObject {
                listen: socks5_inbound.listen.clone(),
                port: socks5_inbound.port,
                protocol: "socks".to_string(),
                tag: "SOCKS_IN".to_string(),
                settings: Some(socks5),
                ..Default::default()
            };
            config.inbounds.push(inbound);
        }
    }
}

fn json_to_outbound(json_str: &str) -> Result<OutboundObject, serde_json::Error> {
    serde_json::from_str(json_str)
}

fn config_to_json(origin_config: &V2RayConfig, outbound_raw: &str) -> Result<serde_json::Value> {
    let mut root = serde_json::to_value(&origin_config)?;

    if root["inbounds"].is_null() {
        return Ok(serde_json::Value::Null);
    };

    let fix_format = |root: &mut serde_json::Value, keys: Vec<&'static str>| {
        keys.into_iter().for_each(|key| {
            if let serde_json::Value::Array(xbounds) = &mut root[key] {
                xbounds.iter_mut().for_each(|xbound| {
                    let protocol = xbound["protocol"]
                        .as_str()
                        .unwrap_or("null")
                        .replace('-', "_");

                    let setting = &mut xbound["settings"][&protocol];
                    xbound["settings"] = setting.clone();
                });
            }
        });
    };

    if outbound_raw.is_empty() {
        fix_format(&mut root, vec!["inbounds", "outbounds"]);
    } else {
        fix_format(&mut root, vec!["inbounds"]);

        let outbound = serde_json::Value::from_str(outbound_raw)?;

        if !outbound.is_null() {
            root["outbounds"] = serde_json::Value::Array(vec![outbound]);
        }
    }

    Ok(root)
}

pub fn generate_config(
    node_data: &core_data::NodeData,
    inbounds: &config_manager::Inbounds,
) -> Result<String> {
    let mut node_config = V2RayConfig::default();
    let mut json;

    set_inbound_object(&mut node_config, inbounds);

    if !node_data.url.contains("://") {
        json = config_to_json(&node_config, &node_data.raw)?;
    } else {
        let mut outbound = json_to_outbound(&node_data.raw)?;

        if outbound.tag.is_empty() {
            outbound.tag = "PROXY".to_string();
        }

        node_config.outbounds.push(outbound);

        json = config_to_json(&node_config, "")?;
    }

    check_is_default_and_delete(&mut json);

    Ok(json.to_string())
}

pub fn generate_config_by_socks(
    node_data: &core_data::NodeData,
    inbound: &config_manager::SOCKS5Inbound,
) -> Result<String> {
    let mut node_config = V2RayConfig::default();
    let mut json;

    set_inbound_socks5_object(&Some(inbound), &mut node_config);

    if !node_data.url.contains("://") {
        json = config_to_json(&node_config, &node_data.raw)?;
    } else {
        let mut outbound = json_to_outbound(&node_data.raw)?;

        if outbound.tag.is_empty() {
            outbound.tag = "PROXY".to_string();
        }

        node_config.outbounds.push(outbound);

        json = config_to_json(&node_config, "")?;
    }

    check_is_default_and_delete(&mut json);

    Ok(json.to_string())
}
