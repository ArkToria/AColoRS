use std::str::FromStr;

use anyhow::Result;
use config_manager::SOCKS5Inbound;
use kernel_manager::{v2ray::raycore::V2RayCore, CoreTool};
use serialize_tool::serialize::serializetool::decode_outbound_from_url;
const INBOUND_STR: &str = r#"
{
    "enable": true,
    "listen": "127.0.0.1",
    "port": 4444,
    "udp_enable": true,
    "auth": {}
}
        "#;
#[test]
fn generate_vmess_socks_config_and_print_json() -> Result<()> {
    let mut core = match V2RayCore::new("v2ray") {
        Ok(c) => c,
        Err(e) => {
            println!("Core Error :{}", e);
            return Ok(());
        }
    };

    let inbounds = SOCKS5Inbound::from_str(INBOUND_STR)?;

    let node_data=decode_outbound_from_url("vmess://eyJhZGQiOiJ0ZXN0MiIsImFpZCI6MzEyLCJob3N0IjoiZmQiLCJpZCI6ImIyOTYxOWI3LTZkOWEtNGQwYy03MjI5LWRkMjczNGExY2FhNCIsIm5ldCI6IndzIiwicGF0aCI6ImFmZCIsInBvcnQiOjE0MiwicHMiOiJ0ZXN0MSIsInNjeSI6ImNoYWNoYTIwLXBvbHkxMzA1Iiwic25pIjoiNDEyIiwidGxzIjoidGxzIiwidHlwZSI6Im5vbmUiLCJ2IjoiMiJ9@");
    let node_data = match node_data {
        Ok(d) => d,
        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    };

    core.set_config_by_node_and_socks_inbound(&node_data, &inbounds)?;

    dbg!(core.get_config());
    println!("{}", core.get_config());

    Ok(())
}
