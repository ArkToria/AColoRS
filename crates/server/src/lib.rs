use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use anyhow::{anyhow, Context};
use core_protobuf::acolors_proto::config_manager_server::ConfigManagerServer;
use core_protobuf::acolors_proto::core_manager_server::CoreManagerServer;
use core_protobuf::acolors_proto::notifications_server::NotificationsServer;
use spdlog::{error, info};
use tokio::sync::{broadcast, RwLock};
use tonic::transport::Server;

use crate::config_manager::{config_read_to_json, AColoRSConfig};
use crate::core_manager::AColoRSCore;
use crate::notifications::AColoRSNotifications;
use crate::profile::AColoRSProfile;
use core_protobuf::acolors_proto::profile_manager_server::ProfileManagerServer;

mod config_manager;
mod core_manager;
mod notifications;
mod profile;
mod signal_stream;
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

const BUFFER_SIZE: usize = 16;
async fn start_server<P: AsRef<Path>>(
    addr: SocketAddr,
    database_path: P,
    core_path: P,
    config_path: P,
) -> Result<()> {
    let mut config = config_read_to_json(&config_path).await?;
    let config_inbounds = match config.get_mut("inbounds") {
        Some(v) => {
            let inbounds = serde_json::from_value(v.take())?;
            Some(inbounds)
        }
        None => None,
    };

    let (signal_sender, _) = broadcast::channel(BUFFER_SIZE);

    let acolors_notifications = AColoRSNotifications::new(signal_sender.clone());

    let profile_task_producer = Arc::new(
        profile_manager::ProfileTaskProducer::new(
            database_path,
            signal_sender.clone(),
            BUFFER_SIZE,
        )
        .await?,
    );

    let acolors_profile = AColoRSProfile::new(profile_task_producer.clone()).await;

    let inbounds = Arc::new(RwLock::new(config_inbounds.unwrap_or_default()));
    let acolors_config =
        AColoRSConfig::new(config_path, inbounds.clone(), signal_sender.clone()).await;

    let mut acolors_core = AColoRSCore::new(profile_task_producer, inbounds, signal_sender);
    acolors_core
        .add_core("v2ray", "default_core", core_path.as_ref().as_os_str())
        .await?;
    acolors_core.set_core("default_core").await?;

    info!("gRPC server is available at http://{}\n", addr);

    Server::builder()
        .add_service(NotificationsServer::new(acolors_notifications))
        .add_service(ProfileManagerServer::new(acolors_profile))
        .add_service(ConfigManagerServer::new(acolors_config))
        .add_service(CoreManagerServer::new(acolors_core))
        .serve(addr)
        .await
        .context("Failed to start gRPC server.")?;
    Ok(())
}

fn check_tcp_bind(bind_address: SocketAddr) -> Result<()> {
    if (TcpListener::bind(&bind_address)).is_err() {
        return Err(anyhow!("Cannot start server on address {}.", bind_address));
    }
    Ok(())
}
