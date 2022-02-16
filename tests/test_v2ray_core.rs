use std::str::FromStr;

use anyhow::Result;
use config_manager::Inbounds;
use kernel_manager::{v2ray::coretool::V2rayCore, CoreTool};
use profile_manager::serialize::serializetool::decode_outbound_from_url;
const INBOUND_STR: &str = r#"
{
    "socks5": {
        "enable": true,
        "listen": "127.0.0.1",
        "port": 4444,
        "udp_enable": true,
        "auth": {}
    },
    "http": {
        "enable": true,
        "listen": "127.0.0.1",
        "port": 4445,
        "auth": {
            "enable": false,
            "username": "testusername",
            "password": "testpassword"
        }
    }
}
        "#;
#[test]
fn generate_vmess_config_and_print_json() -> Result<()> {
    let mut core = V2rayCore::new("v2ray")?;

    let inbounds = Inbounds::from_str(INBOUND_STR)?;

    let node_data=decode_outbound_from_url("vmess://eyJhZGQiOiJ0ZXN0MiIsImFpZCI6MzEyLCJob3N0IjoiZmQiLCJpZCI6ImIyOTYxOWI3LTZkOWEtNGQwYy03MjI5LWRkMjczNGExY2FhNCIsIm5ldCI6IndzIiwicGF0aCI6ImFmZCIsInBvcnQiOjE0MiwicHMiOiJ0ZXN0MSIsInNjeSI6ImNoYWNoYTIwLXBvbHkxMzA1Iiwic25pIjoiNDEyIiwidGxzIjoidGxzIiwidHlwZSI6Im5vbmUiLCJ2IjoiMiJ9@");
    let node_data = match node_data {
        Ok(d) => d,
        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    };

    let config = V2rayCore::generate_config(&node_data, &inbounds)?;

    core.set_config(config)?;

    dbg!(core.get_config());
    println!("{}", core.get_config());

    Ok(())
}
