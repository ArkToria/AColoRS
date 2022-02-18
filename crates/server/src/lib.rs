use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use anyhow::{anyhow, Context};
use core_protobuf::acolors_proto::config_manager_server::ConfigManagerServer;
use core_protobuf::acolors_proto::core_manager_server::CoreManagerServer;
use kernel_manager::v2ray::coretool::V2RayCore;
use kernel_manager::CoreTool;
use spdlog::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, RwLock};
use tonic::transport::Server;

use crate::core_manager::AColoRSCore;
use crate::inbounds::AColoRSConfig;
use crate::profile::AColoRSProfile;
use core_protobuf::acolors_proto::profile_manager_server::ProfileManagerServer;

mod core_manager;
mod inbounds;
mod profile;
mod utils;

pub fn serve<P: AsRef<Path>>(
    address: SocketAddr,
    database_path: P,
    core_path: P,
    config_path: P,
) -> Result<()> {
    check_tcp_bind(address)?;

    let addr: SocketAddr = address;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not build tokio runtime");

    match rt.block_on(start_server(addr, database_path, core_path, config_path)) {
        Ok(()) => {
            info!("gRPC Server stopped normally.");
        }
        Err(e) => {
            error!("gRPC Server error: {}", e);
        }
    };

    Ok(())
}

async fn start_server<P: AsRef<Path>>(
    addr: SocketAddr,
    database_path: P,
    core_path: P,
    config_path: P,
) -> Result<()> {
    let mut config = config_read_to_json(config_path).await?;
    let config_inbounds = match config.get_mut("inbounds") {
        Some(v) => {
            let inbounds = serde_json::from_value(v.take())?;
            Some(inbounds)
        }
        None => None,
    };

    let profile_task_producer =
        Arc::new(profile_manager::ProfileTaskProducer::new(database_path).await?);
    let acolors_profile = AColoRSProfile::new(profile_task_producer.clone()).await;

    let inbounds = Arc::new(RwLock::new(config_inbounds.unwrap_or_default()));
    let acolors_config = AColoRSConfig::new(inbounds.clone()).await;

    let core = match V2RayCore::new(core_path.as_ref().as_os_str()) {
        Ok(c) => {
            info!("Core <{}> version ({})", c.get_name(), c.get_version());
            c
        }
        Err(e) => {
            error!("Core not found : {}", e);
            std::process::exit(1);
        }
    };
    let wraped_core = Arc::new(Mutex::new(core));
    let acolors_core = AColoRSCore::new::<V2RayCore>(wraped_core, profile_task_producer, inbounds);

    info!("gRPC server is available at http://{}\n", addr);

    Server::builder()
        .add_service(ProfileManagerServer::new(acolors_profile))
        .add_service(ConfigManagerServer::new(acolors_config))
        .add_service(CoreManagerServer::new(acolors_core))
        .serve(addr)
        .await
        .context("Failed to start gRPC server.")?;
    Ok(())
}

async fn config_read_to_json<P: AsRef<Path>>(config_path: P) -> Result<serde_json::Value> {
    let mut file = read_or_create(config_path).await?;

    let mut content = String::new();
    file.read_to_string(&mut content).await?;

    let v = serde_json::from_str(&content)?;

    Ok(v)
}
const DEFAULT_CONFIG_FILE_CONTENT: &str = r#"{
  "inbounds": {
    "socks5": {
      "enable": true,
      "listen": "127.0.0.1",
      "port": 4444,
      "udp_enable": true
    },
    "http": {
      "enable": true,
      "listen": "127.0.0.1",
      "port": 4445
    }
  }
}"#;
async fn read_or_create<P: AsRef<Path>>(config_path: P) -> Result<tokio::fs::File> {
    let file = match tokio::fs::File::open(&config_path).await {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                create_and_reopen(config_path).await?
            } else {
                return Err(e.into());
            }
        }
    };
    Ok(file)
}

async fn create_and_reopen<P: AsRef<Path>>(config_path: P) -> Result<tokio::fs::File> {
    let dir = match config_path.as_ref().parent() {
        Some(d) => d,
        None => {
            return Err(anyhow!("No Directory"));
        }
    };
    tokio::fs::create_dir_all(dir).await?;
    let mut f = tokio::fs::File::create(&config_path).await?;
    f.write_all(DEFAULT_CONFIG_FILE_CONTENT.as_bytes()).await?;
    Ok(tokio::fs::File::open(&config_path).await?)
}

fn check_tcp_bind(bind_address: SocketAddr) -> Result<()> {
    if (TcpListener::bind(&bind_address)).is_err() {
        return Err(anyhow!("Cannot start server on address {}.", bind_address));
    }
    Ok(())
}
