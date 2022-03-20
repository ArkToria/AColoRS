use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use acolors_signal::send_or_warn_print;
use core_protobuf::acolors_proto::{config_manager_server::ConfigManager, *};
use serialize_tool::serialize::serializer::check_is_default_and_delete;
use spdlog::debug;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{broadcast, RwLock},
};
use tonic::{Request, Response, Status};

type InboundsLock = Arc<RwLock<config_manager::Inbounds>>;
#[derive(Debug)]
pub struct AColoRSConfig {
    inbounds: InboundsLock,
    path: PathBuf,
    signal_sender: broadcast::Sender<profile_manager::AColorSignal>,
}

impl AColoRSConfig {
    pub fn new<P: AsRef<Path>>(
        path: P,
        inbounds: InboundsLock,
        signal_sender: broadcast::Sender<profile_manager::AColorSignal>,
    ) -> Self {
        let path = path.as_ref().to_path_buf();
        Self {
            inbounds,
            path,
            signal_sender,
        }
    }
}

#[tonic::async_trait]
impl ConfigManager for AColoRSConfig {
    async fn set_inbounds(
        &self,
        request: Request<Inbounds>,
    ) -> Result<Response<SetInboundsReply>, Status> {
        debug!("Set inbounds from {:?}", request.remote_addr());
        debug!("Current inbounds: {:?}", &*self.inbounds.read().await);
        let mut inbounds_write = self.inbounds.write().await;

        let inbounds = request.into_inner();

        write_config_to_file(&self.path, &inbounds).await?;
        *inbounds_write = inbounds.into();

        send_or_warn_print(
            &self.signal_sender,
            acolors_signal::AColorSignal::UpdateInbounds,
        );

        let reply = SetInboundsReply {};
        Ok(Response::new(reply))
    }
    async fn get_inbounds(
        &self,
        request: Request<GetInboundsRequest>,
    ) -> Result<Response<Inbounds>, Status> {
        debug!("Set inbounds from {:?}", request.remote_addr());
        let inbounds_read = self.inbounds.read().await;

        let inbounds = &*inbounds_read;

        let inbounds = inbounds.clone();

        let reply = inbounds.into();
        Ok(Response::new(reply))
    }
}

pub async fn write_config_to_file<P: AsRef<Path>>(
    path: P,
    inbounds: &Inbounds,
) -> std::io::Result<()> {
    let mut config = config_read_to_json(&path).await?;
    let mut file = tokio::fs::File::create(&path).await?;

    let inbounds_c = config_manager::Inbounds::from(inbounds.clone());
    match config.get_mut("inbounds") {
        Some(v) => {
            *v = serde_json::to_value(inbounds_c)?;
        }
        None => {
            if let Some(v) = config.as_object_mut() {
                v.insert("inbounds".to_string(), serde_json::to_value(inbounds_c)?);
            }
        }
    }

    check_is_default_and_delete(&mut config);

    let content = serde_json::to_string_pretty(&config)?;

    debug!("{}", &content);

    file.write_all(content.as_bytes()).await?;
    file.flush().await?;
    file.sync_all().await?;

    Ok(())
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
pub async fn config_read_to_json<P: AsRef<Path>>(
    config_path: P,
) -> std::io::Result<serde_json::Value> {
    let mut content = String::new();
    if let Ok(mut file) = tokio::fs::File::open(&config_path).await {
        file.read_to_string(&mut content).await?;
    }

    if content.is_empty() {
        let mut file = open_or_create(config_path).await?;
        file.write_all(DEFAULT_CONFIG_FILE_CONTENT.as_bytes())
            .await?;
        content = DEFAULT_CONFIG_FILE_CONTENT.to_string();
    }

    let v = serde_json::from_str(&content)?;

    Ok(v)
}

async fn open_or_create<P: AsRef<Path>>(config_path: P) -> std::io::Result<tokio::fs::File> {
    if let Some(parent) = config_path.as_ref().parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&config_path)
        .await
}
