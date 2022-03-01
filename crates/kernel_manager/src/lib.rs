mod core;
pub mod naiveproxy;
pub mod shadowsocks;
pub mod trojan_go;
pub mod v2ray;

use std::ffi::OsStr;

use naiveproxy::coretool::NaiveProxy;
use shadowsocks::coretool::Shadowsocks;
use trojan_go::coretool::TrojanGo;
use v2ray::coretool::V2RayCore;

pub use crate::core::CoreTool;

type Core = dyn CoreTool + Sync + Send + 'static;
pub fn create_core_by_path<S: AsRef<OsStr> + ?Sized>(
    path: &S,
    core_type: &str,
) -> anyhow::Result<Box<Core>> {
    match core_type.to_ascii_lowercase().as_str() {
        "v2ray" => Ok(Box::new(V2RayCore::new(path)?) as Box<Core>),
        "shadowsocks" => Ok(Box::new(Shadowsocks::new(path)?) as Box<Core>),
        "trojan-go" => Ok(Box::new(TrojanGo::new(path)?) as Box<Core>),
        "naiveproxy" => Ok(Box::new(NaiveProxy::new(path)?) as Box<Core>),
        _ => Err(anyhow::anyhow!("Core Not Implemented")),
    }
}
