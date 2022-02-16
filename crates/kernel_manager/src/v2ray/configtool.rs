use core_protobuf::v2ray_proto::*;
use core_protobuf::v2ray_proto::{
    inbound_object::{inbound_settings, InboundSettings},
    InboundObject, V2RayConfig,
};

pub fn set_inbound_object(config: &mut V2RayConfig, inbounds: &config_manager::Inbounds) {
    set_inbound_http_object(inbounds, config);
    set_inbound_socks5_object(inbounds, config);
}

fn set_inbound_http_object(inbounds: &config_manager::Inbounds, config: &mut V2RayConfig) {
    let http_settings = &inbounds.http;
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

fn set_inbound_socks5_object(inbounds: &config_manager::Inbounds, config: &mut V2RayConfig) {
    let socks5_settings = &inbounds.socks5;
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
