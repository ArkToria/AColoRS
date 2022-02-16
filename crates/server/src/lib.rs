use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

use anyhow::Result;
use anyhow::{anyhow, Context};
use core_protobuf::acolors_proto::config_manager_server::ConfigManagerServer;
use spdlog::{error, info};
use tokio::sync::RwLock;
use tonic::transport::Server;

use crate::inbounds::AColoRSConfig;
use crate::profile::AColoRSProfile;
use core_protobuf::acolors_proto::profile_manager_server::ProfileManagerServer;

mod inbounds;
mod profile;
mod utils;

pub fn serve(address: SocketAddr, database_path: String) -> Result<()> {
    check_tcp_bind(address)?;

    let addr: SocketAddr = address;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not build tokio runtime");

    match rt.block_on(start_server(addr, database_path)) {
        Ok(()) => {
            info!("gRPC Server stopped normally.");
        }
        Err(e) => {
            error!("gRPC Server error: {}", e);
        }
    };

    Ok(())
}

async fn start_server(addr: SocketAddr, database_path: String) -> Result<()> {
    let profile_task_producer =
        Arc::new(profile_manager::ProfileTaskProducer::new(database_path).await?);
    let acolors_profile = AColoRSProfile::new(profile_task_producer).await;
    let inbounds = Arc::new(RwLock::new(config_manager::Inbounds::default()));
    let acolors_config = AColoRSConfig::new(inbounds).await;

    info!("gRPC server is available at http://{}\n", addr);

    Server::builder()
        .add_service(ProfileManagerServer::new(acolors_profile))
        .add_service(ConfigManagerServer::new(acolors_config))
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
