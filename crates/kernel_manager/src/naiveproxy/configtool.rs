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
        config.push_str(&format!(
            " --listen=socks://{}:{}",
            socks.listen, socks.port
        ));
        Ok(())
    }
    let mut config = String::new();

    if inbounds.http.is_some() {
        warn!("NaiveProxy currently don't have http inbounds.");
    }
    set_inbounds(inbounds, &mut config)?;

    let protocol = node_data.url.split("://").next().unwrap_or("");
    match protocol {
        "naive+https" | "naive+quic" => {
            config.push_str(&format!(
                " --proxy={}",
                &node_data.url[6..node_data.url.len()]
            ));
        }
        _ => {
            return Err(anyhow!("Protocol Error: {}", protocol));
        }
    };

    Ok(config)
}
