impl From<crate::Inbounds> for core_protobuf::acolors_proto::Inbounds {
    fn from(inbounds: crate::Inbounds) -> Self {
        let socks5 = inbounds.socks5.map(|so5| {
            let auth = so5
                .auth
                .map(|au| core_protobuf::acolors_proto::inbounds::Auth {
                    enable: au.enable,
                    username: au.username,
                    password: au.password,
                });
            core_protobuf::acolors_proto::inbounds::Socks5 {
                enable: so5.enable,
                listen: so5.listen,
                port: so5.port,
                udp_enable: so5.udp_enable,
                udp_ip: so5.udp_ip,
                user_level: so5.user_level,
                auth,
            }
        });
        let http = inbounds.http.map(|ht| {
            let auth = ht
                .auth
                .map(|au| core_protobuf::acolors_proto::inbounds::Auth {
                    enable: au.enable,
                    username: au.username,
                    password: au.password,
                });
            core_protobuf::acolors_proto::inbounds::Http {
                enable: ht.enable,
                listen: ht.listen,
                port: ht.port,
                user_level: ht.user_level,
                allow_transparent: ht.allow_transparent,
                timeout: ht.timeout,
                auth,
            }
        });
        core_protobuf::acolors_proto::Inbounds { socks5, http }
    }
}

impl From<core_protobuf::acolors_proto::Inbounds> for crate::Inbounds {
    fn from(inbounds: core_protobuf::acolors_proto::Inbounds) -> Self {
        let socks5 = inbounds.socks5.map(|so5| {
            let auth = so5.auth.map(|au| crate::inbound::Auth {
                enable: au.enable,
                username: au.username,
                password: au.password,
            });
            crate::inbound::SOCKS5Inbound {
                enable: so5.enable,
                listen: so5.listen,
                port: so5.port,
                udp_enable: so5.udp_enable,
                udp_ip: so5.udp_ip,
                user_level: so5.user_level,
                auth,
            }
        });
        let http = inbounds.http.map(|ht| {
            let auth = ht.auth.map(|au| crate::inbound::Auth {
                enable: au.enable,
                username: au.username,
                password: au.password,
            });
            crate::inbound::HTTPInbound {
                enable: ht.enable,
                listen: ht.listen,
                port: ht.port,
                user_level: ht.user_level,
                allow_transparent: ht.allow_transparent,
                timeout: ht.timeout,
                auth,
            }
        });
        crate::Inbounds { socks5, http }
    }
}
