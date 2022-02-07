use std::net::{SocketAddr, TcpListener};

use anyhow::Result;
use anyhow::{anyhow, Context};
use spdlog::{error, info};
use tonic::transport::Server;

use crate::profile::AColoRSProfile;
use crate::protobuf::acolors_proto::profile_manager_server::ProfileManagerServer;

mod convert;
mod profile;
mod protobuf;
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
    let acolors_profile = AColoRSProfile::new(database_path).await?;

    info!("gRPC server is available at http://{}\n", addr);

    Server::builder()
        .add_service(ProfileManagerServer::new(acolors_profile))
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
