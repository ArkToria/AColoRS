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
            " --listen=socks://{}:{}",
            socks.listen, socks.port
        ));
        Ok(())
    }
    let mut config = String::new();

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
