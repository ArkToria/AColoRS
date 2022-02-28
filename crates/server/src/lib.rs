use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::sync::Arc;

use ::config_manager::CoreList;
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

pub fn serve<P: AsRef<Path>>(
    address: SocketAddr,
    database_path: P,
    core_path: P,
    core_name: &str,
    config_path: P,
) -> Result<()> {
    check_tcp_bind(address)?;

    let addr: SocketAddr = address;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not build tokio runtime");

    match rt.block_on(start_server(
        addr,
        database_path,
        core_path,
        core_name,
        config_path,
    )) {
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
    core_name: &str,
    config_path: P,
) -> Result<()> {
    let (acolors_notifications, acolors_profile, acolors_config, acolors_core) =
        create_services(database_path, config_path, core_name, core_path).await?;

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

async fn create_services<P: AsRef<Path>>(
    database_path: P,
    config_path: P,
    core_name: &str,
    core_path: P,
) -> Result<(
    AColoRSNotifications,
    AColoRSProfile,
    AColoRSConfig,
    AColoRSCore,
)> {
    let mut config = config_read_to_json(&config_path).await?;
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
    let config_inbounds = config
        .get_mut("inbounds")
        .map(|v| serde_json::from_value(v.take()))
        .transpose()?;
    let inbounds = Arc::new(RwLock::new(config_inbounds.unwrap_or_default()));
    let acolors_config = AColoRSConfig::new(config_path, inbounds.clone(), signal_sender.clone());
    let mut acolors_core = AColoRSCore::create(
        profile_task_producer.clone(),
        inbounds.clone(),
        signal_sender,
    )
    .await;

    let acolors_profile = AColoRSProfile::new(profile_task_producer, inbounds);

    let cores_value = config.get_mut("cores");
    add_cores(cores_value, &mut acolors_core, core_name, core_path).await?;

    acolors_core.set_core("default_core").await?;
    Ok((
        acolors_notifications,
        acolors_profile,
        acolors_config,
        acolors_core,
    ))
}

async fn add_cores<P: AsRef<Path>>(
    cores_value: Option<&mut serde_json::Value>,
    acolors_core: &mut AColoRSCore,
    core_name: &str,
    core_path: P,
) -> Result<()> {
    acolors_core
        .add_core(core_name, "default_core", core_path.as_ref().as_os_str())
        .await?;

    if let Some(cores) = cores_value {
        let mut cores_object = serde_json::Value::Object(serde_json::Map::new());
        cores_object
            .as_object_mut()
            .unwrap()
            .insert("cores".to_string(), cores.take());

        let config_inbounds: CoreList = serde_json::from_value(cores_object)?;

        for core in config_inbounds.cores {
            acolors_core
                .add_core(&core.name, &core.tag, &core.path)
                .await?;
        }
    }
    Ok(())
}

fn check_tcp_bind(bind_address: SocketAddr) -> Result<()> {
    if (TcpListener::bind(&bind_address)).is_err() {
        return Err(anyhow!("Cannot start server on address {}.", bind_address));
    }
    Ok(())
}
