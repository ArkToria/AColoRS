use anyhow::{anyhow, Result};

use crate::regex;
use core_data::NodeData;

pub fn naiveproxy_outbound_from_url(url_str: String) -> Result<NodeData> {
    // url scheme:
    // naive+https://username:password@hostname:port?padding=true#tag
    // naive+quic://username:password@hostname:port?padding=false#tag
    let re = regex!(r#"(\w+)://([^/:]*):([^:@]*)@([^:]*):([^:\?]*)([\?]*)([^#]*)#([^#]*)"#);
    // regex::Regex::new(r#"(\w+)://([^/:]*):([^:@]*)@([^:]*):([^:\?]*)([\?]*)([^#]*)#([^#]*)"#)?;
    let caps = re.captures(&url_str).ok_or_else(|| {
        dbg!(&url_str);
        anyhow!("Failed to parse naive url")
    })?;

    if caps.len() < 8 {
        return Err(anyhow!("Parse naiveproxy url error"));
    }

    /*
    let _padding = caps[7]
        .split('=')
        .last()
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(false);
    */

    let mut node = NodeData::default();

    let name = urlencoding::decode(&caps[8])?.to_string();

    node.protocol = "naiveproxy".into();
    node.name = name;
    node.address = caps[4].to_string();
    node.port = caps[5].parse()?;
    node.password = caps[3].to_string();
    node.url = url_str;

    Ok(node)
}
