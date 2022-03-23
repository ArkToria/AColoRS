use std::str::FromStr;

use anyhow::{anyhow, Result};
pub fn generate_config(
    node_data: &core_data::NodeData,
    inbounds: &config_manager::SOCKS5Inbound,
) -> Result<String> {
    fn set_inbounds(
        inbounds: &config_manager::SOCKS5Inbound,
        config: &mut String,
    ) -> Result<(), anyhow::Error> {
        let socks = inbounds;

        config.push_str(&format!(
            " -url-option listen={}:{}",
            socks.listen, socks.port
        ));
        Ok(())
    }
    let mut config = String::new();

    set_inbounds(inbounds, &mut config)?;

    let mut split_url = node_data.url.split("://");
    let protocol = split_url.next().unwrap_or("");
    let content = split_url.next().unwrap_or("");
    match protocol {
        "trojan" | "trojan-go" => {
            config.push_str(&format!(" -url trojan-go://{}", content));
        }
        "" => {
            let json = serde_json::Value::from_str(&node_data.raw)?;
            add_outbound(&json, &mut config, &node_data.name)?;
        }
        _ => {
            return Err(anyhow!("Protocol Error: {}", protocol));
        }
    };

    Ok(config)
}

fn add_outbound(
    json: &serde_json::Value,
    config: &mut String,
    name: &str,
) -> Result<(), anyhow::Error> {
    let protocol = protocol_str(json)?;
    match protocol {
        "trojan" => add_trojan_outbound(json, protocol, config, name),
        "trojan-go" => add_trojan_go_outbound(),
        _ => Err(anyhow!("Protocol Error: {}", protocol)),
    }
}

fn add_trojan_go_outbound() -> Result<(), anyhow::Error> {
    Err(anyhow!("Trojan-go unimplemented"))
}

fn add_trojan_outbound(
    outbound: &serde_json::Value,
    protocol: &str,
    config: &mut String,
    name: &str,
) -> Result<(), anyhow::Error> {
    let server = get_server(outbound, protocol)?;
    let address = get_address(server)?;
    let port = get_port(server)?;
    let password = get_password(server)?;
    let stream_settings = get_stream_settings(outbound)?;
    let network = get_network(stream_settings)?;
    let mut flag = false;
    let mut url = format!(" -url trojan-go://{}@{}:{}/?", password, address, port);
    if let Ok(settings) = get_tls_settings(stream_settings) {
        let sni = get_sni(settings)?;
        flag = true;
        url.push_str(&format!("sni={}", sni));
    }
    match network {
        "original" => {}
        "ws" => {
            let path;
            let mut host = "";
            if let Ok(settings) = get_ws_settings(stream_settings) {
                path = get_path(settings)?;
                host = get_ws_host(settings)?;
            } else {
                path = "/";
            };
            if flag {
                url.push('&');
            }
            url.push_str(&format!("type=ws&path={}", path));
            if !host.is_empty() {
                url.push_str(&format!("&host={}", host))
            }
        }
        "h2" => {
            let settings = get_h2_settings(stream_settings)?;
            let path = get_path(settings)?;
            let hosts = get_h2_hosts(settings)?;
            let mut host_result = String::new();

            for host in hosts {
                host_result.push_str(host);
                host_result.push(',');
            }

            let host_result = &host_result[0..host_result.len() - 1];

            if flag {
                url.push('&');
            }
            url.push_str(&format!(
                "type=h2&path={}&host={}",
                path,
                urlencoding::encode(host_result),
            ));
        }
        _ => {
            return Err(anyhow!("No Such network: {}", network));
        }
    }
    url.push_str(&format!("#{}", name));
    config.push_str(&url);
    Ok(())
}
fn get_h2_hosts(settings: &serde_json::Value) -> Result<Vec<&str>, anyhow::Error> {
    let option_hosts = settings
        .get("host")
        .unwrap_or(&serde_json::Value::Null)
        .as_array();
    let mut result = Vec::new();
    if let Some(hosts) = option_hosts {
        for host in hosts {
            let host = host
                .as_str()
                .ok_or_else(|| anyhow!("Host element should be a String"))?;
            result.push(host);
        }
    }

    Ok(result)
}
fn get_h2_settings(
    stream_settings: &serde_json::Value,
) -> Result<&serde_json::Value, anyhow::Error> {
    stream_settings
        .get("httpSettings")
        .ok_or_else(|| anyhow!("No httpSettings"))
}
fn get_ws_host(settings: &serde_json::Value) -> Result<&str, anyhow::Error> {
    Ok(settings
        .get("headers")
        .map(|headers| {
            headers
                .get("host")
                .map(|s| s.as_str().unwrap_or(""))
                .unwrap_or("")
        })
        .unwrap_or(""))
}
fn get_path(settings: &serde_json::Value) -> Result<&str, anyhow::Error> {
    Ok(settings
        .get("path")
        .map(|path| path.as_str().unwrap_or("/"))
        .unwrap_or("/"))
}
fn get_ws_settings(
    stream_settings: &serde_json::Value,
) -> Result<&serde_json::Value, anyhow::Error> {
    stream_settings
        .get("wsSettings")
        .ok_or_else(|| anyhow!("No wsSettings"))
}
fn get_sni(settings: &serde_json::Value) -> Result<&str, anyhow::Error> {
    settings
        .get("serverName")
        .ok_or_else(|| anyhow!("No sni"))?
        .as_str()
        .ok_or_else(|| anyhow!("sni should be a String"))
}
fn get_tls_settings(
    stream_settings: &serde_json::Value,
) -> Result<&serde_json::Value, anyhow::Error> {
    stream_settings
        .get("tlsSettings")
        .ok_or_else(|| anyhow!("No tlsSettings"))
}
fn get_network(settings: &serde_json::Value) -> Result<&str, anyhow::Error> {
    Ok(
        match settings
            .get("network")
            .ok_or_else(|| anyhow!("No Network"))?
            .as_str()
            .ok_or_else(|| anyhow!("Network should be a String"))?
        {
            "tcp" => "original",
            "ws" => "ws",
            "http" => "h2",
            _ => return Err(anyhow!("No transport existed")),
        },
    )
}

fn get_stream_settings(outbound: &serde_json::Value) -> Result<&serde_json::Value, anyhow::Error> {
    outbound
        .get("streamSettings")
        .ok_or_else(|| anyhow!("No streamSettings"))
}

fn get_password(server: &serde_json::Value) -> Result<&str, anyhow::Error> {
    server
        .get("password")
        .ok_or_else(|| anyhow!("No Password"))?
        .as_str()
        .ok_or_else(|| anyhow!("Password should be a String"))
}

fn get_port(server: &serde_json::Value) -> Result<u64, anyhow::Error> {
    server
        .get("port")
        .ok_or_else(|| anyhow!("No Port"))?
        .as_u64()
        .ok_or_else(|| anyhow!("Port should be an integer"))
}

fn get_address(server: &serde_json::Value) -> Result<&str, anyhow::Error> {
    server
        .get("address")
        .ok_or_else(|| anyhow!("No Address"))?
        .as_str()
        .ok_or_else(|| anyhow!("Address should be a String"))
}

fn get_server<'a>(
    outbound: &'a serde_json::Value,
    trojan: &str,
) -> Result<&'a serde_json::Value, anyhow::Error> {
    outbound
        .get("settings")
        .ok_or_else(|| anyhow!("No trojan-go settings"))?
        .get(trojan)
        .ok_or_else(|| anyhow!("No trojan-go settings"))?
        .get("servers")
        .ok_or_else(|| anyhow!("No trojan-go servers"))?
        .as_array()
        .ok_or_else(|| anyhow!("Servers should be an array"))?
        .get(0)
        .ok_or_else(|| anyhow!("Servers array is empty"))
}

fn protocol_str(outbound: &serde_json::Value) -> Result<&str, anyhow::Error> {
    outbound
        .get("protocol")
        .ok_or_else(|| anyhow!("No protocol specified"))?
        .as_str()
        .ok_or_else(|| anyhow!("Protocol should be a string"))
}
