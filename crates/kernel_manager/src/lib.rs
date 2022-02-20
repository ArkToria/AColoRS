mod core;
pub mod v2ray;

use std::ffi::OsStr;

use v2ray::coretool::V2RayCore;

pub use crate::core::CoreTool;

type Core = dyn CoreTool + Sync + Send + 'static;
pub fn create_core_by_path<S: AsRef<OsStr> + ?Sized>(
    path: &S,
    core_type: &str,
) -> anyhow::Result<Box<Core>> {
    match core_type.to_ascii_lowercase().as_str() {
        "v2ray" => Ok(Box::new(V2RayCore::new(path)?) as Box<Core>),
        _ => {
            todo!()
        }
    }
}
