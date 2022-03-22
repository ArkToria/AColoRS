use std::str::FromStr;

use anyhow::{anyhow, Result};
use spdlog::warn;

pub fn generate_config(
    node_data: &core_data::NodeData,
    inbounds: &config_manager::Inbounds,
) -> Result<String> {
    fn set_inbounds(
        inbounds: &config_manager::Inbounds,
        config: &mut String,
    ) -> Result<(), anyhow::Error> {
        let socks = inbounds
            .socks5
            .as_ref()
            .ok_or_else(|| anyhow!("Socks inbound not found"))?;
        config.push_str(&format!(" --local-addr {}:{}", socks.listen, socks.port));
        Ok(())
    }
    let mut config = String::new();

    if inbounds.http.is_some() {
        warn!("Shadowsocks currently don't have http inbounds.");
    }
    set_inbounds(inbounds, &mut config)?;

    let protocol = node_data.url.split("://").next().unwrap_or("");
    match protocol {
        "ss" => {
            config.push_str(&format!(" --server-url {}", node_data.url));
        }
        "" => {
            let json = serde_json::Value::from_str(&node_data.raw)?;
            add_outbound(&json, &mut config)?;
        }
        _ => {
            return Err(anyhow!("Protocol Error: {}", protocol));
        }
    };

    Ok(config)
}
fn add_outbound(json: &serde_json::Value, config: &mut String) -> Result<(), anyhow::Error> {
    let protocol = protocol_str(json)?;
    if protocol != "shadowsocks" {
        warn!("Protocol Error: {}", protocol)
    };
    let server = get_server(json)?;
    let address = server
        .get("address")
        .ok_or_else(|| anyhow!("No Address"))?
        .as_str()
        .ok_or_else(|| anyhow!("Address should be a String"))?;
    let port = server
        .get("port")
        .ok_or_else(|| anyhow!("No Port"))?
        .as_u64()
        .ok_or_else(|| anyhow!("Port should be an integer"))?;
    let method = server
        .get("method")
        .ok_or_else(|| anyhow!("No Method"))?
        .as_str()
        .ok_or_else(|| anyhow!("Method should be a String"))?;
    let password = server
        .get("password")
        .ok_or_else(|| anyhow!("No Password"))?
        .as_str()
        .ok_or_else(|| anyhow!("Password should be a String"))?;
    config.push_str(&format!(
        " --server-addr {}:{} --encrypt-method {} --password {}",
        address, port, method, password
    ));
    Ok(())
}

fn get_server(outbound: &serde_json::Value) -> Result<&serde_json::Value, anyhow::Error> {
    outbound
        .get("settings")
        .ok_or_else(|| anyhow!("No shadowsocks settings"))?
        .get("shadowsocks")
        .ok_or_else(|| anyhow!("No shadowsocks settings"))?
        .get("servers")
        .ok_or_else(|| anyhow!("No shadowsocks servers"))?
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
