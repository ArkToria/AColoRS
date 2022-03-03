use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::sync::Arc;

use ::config_manager::CoreList;
use acolors_signal::send_or_warn_print;
use anyhow::anyhow;
use anyhow::Result;
use core_protobuf::acolors_proto::config_manager_server::ConfigManagerServer;
use core_protobuf::acolors_proto::core_manager_server::CoreManagerServer;
use core_protobuf::acolors_proto::manager_server::ManagerServer;
use core_protobuf::acolors_proto::notifications_server::NotificationsServer;
use futures::{FutureExt, TryFutureExt};
use server_manager::AColoRSManager;
use spdlog::{error, info};
use sqlx::{Connection, SqliteConnection};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::{join, select};
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
mod server_manager;
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
struct AColoRSServices {
    pub notifications: AColoRSNotifications,
    pub profile: AColoRSProfile,
    pub config: AColoRSConfig,
    pub core: AColoRSCore,
    pub manager: AColoRSManager,
}
const BUFFER_SIZE: usize = 16;
async fn start_server<P: AsRef<Path>>(
    addr: SocketAddr,
    database_path: P,
    core_path: P,
    core_name: &str,
    config_path: P,
) -> Result<()> {
    let (signal_sender, _) = broadcast::channel(BUFFER_SIZE);
    let (notify_shutdown, mut notify_shutdown_rx) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);
    let services = create_services(
        database_path,
        config_path,
        core_name,
        core_path,
        notify_shutdown,
        signal_sender.clone(),
    )
    .await?;

    info!("gRPC server is available at http://{}\n", addr);
    let server = Server::builder()
        .add_service(NotificationsServer::new(services.notifications))
        .add_service(ProfileManagerServer::new(services.profile))
        .add_service(ConfigManagerServer::new(services.config))
        .add_service(CoreManagerServer::new(services.core))
        .add_service(ManagerServer::new(services.manager))
        .serve_with_shutdown(
            addr,
            shutdown_complete_rx.recv().map(|x| x.unwrap_or_default()),
        )
        .unwrap_or_else(|e| error!("Failed to start gRPC server: {}", e));
    let shutdown = async {
        select! {
            _ = tokio::signal::ctrl_c()
            .unwrap_or_else(|e| error!("Failed to listen for Ctrl+C: {}", e)) => info!("Ctrl+C Received"),
            _ = notify_shutdown_rx.recv() => info!("Shutting down"),
        };
        send_or_warn_print(&signal_sender, acolors_signal::AColorSignal::Shutdown);
        shutdown_complete_tx
            .send(())
            .await
            .unwrap_or_else(|e| error!("Shutdown Error: {}", e));
    };
    join!(server, shutdown);
    Ok(())
}

async fn create_services<P: AsRef<Path>>(
    database_path: P,
    config_path: P,
    core_name: &str,
    core_path: P,
    shutdown_sender: broadcast::Sender<()>,
    signal_sender: broadcast::Sender<acolors_signal::AColorSignal>,
) -> Result<AColoRSServices> {
    let mut config = config_read_to_json(&config_path).await?;
    let acolors_notifications = AColoRSNotifications::new(signal_sender.clone());
    let profile = Arc::new(
        profile_manager::Profile::create(
            SqliteConnection::connect(&format!(
                "sqlite://{}",
                database_path.as_ref().as_os_str().to_string_lossy()
            ))
            .await?,
        )
        .await?,
    );
    let config_inbounds = config
        .get_mut("inbounds")
        .map(|v| serde_json::from_value(v.take()))
        .transpose()?;
    let inbounds = Arc::new(RwLock::new(config_inbounds.unwrap_or_default()));
    let acolors_config = AColoRSConfig::new(config_path, inbounds.clone(), signal_sender.clone());
    let mut acolors_core =
        AColoRSCore::create(profile.clone(), inbounds.clone(), signal_sender.clone()).await;

    let acolors_profile = AColoRSProfile::new(profile, inbounds, signal_sender);

    let acolors_manager = AColoRSManager::new(shutdown_sender);

    let cores_value = config.get_mut("cores");
    add_cores(cores_value, &mut acolors_core, core_name, core_path).await?;

    acolors_core.set_core("default_core").await?;
    Ok(AColoRSServices {
        notifications: acolors_notifications,
        profile: acolors_profile,
        config: acolors_config,
        core: acolors_core,
        manager: acolors_manager,
    })
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
