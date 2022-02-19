use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use acolors_signal::send_or_error_print;
use core_protobuf::acolors_proto::{config_manager_server::ConfigManager, *};
use serialize_tool::serialize::serializer::check_is_default_and_delete;
use spdlog::{debug, info};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{broadcast, RwLock},
};
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct AColoRSConfig {
    inbounds: Arc<RwLock<config_manager::Inbounds>>,
    path: PathBuf,
    signal_sender: broadcast::Sender<profile_manager::AColorSignal>,
}

impl AColoRSConfig {
    pub async fn new<P: AsRef<Path>>(
        path: P,
        inbounds: Arc<RwLock<config_manager::Inbounds>>,
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
        info!("Set inbounds from {:?}", request.remote_addr());
        debug!("Current inbounds: {:?}", &*self.inbounds.read().await);
        let mut inbounds_write = self.inbounds.write().await;

        let inbounds = request.into_inner();

        write_config_to_file(&self.path, &inbounds).await?;
        *inbounds_write = inbounds.into();

        send_or_error_print(
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
        info!("Set inbounds from {:?}", request.remote_addr());
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
) -> Result<(), Status> {
    let mut file = match open_or_create(&path).await {
        Ok(f) => f,
        Err(e) => {
            return Err(Status::unavailable(e.to_string()));
        }
    };

    let inbounds_c = config_manager::Inbounds::from(inbounds.clone());
    let mut config = config_read_to_json(&path).await?;
    if let Some(v) = config.get_mut("inbounds") {
        *v = match serde_json::to_value(inbounds_c) {
            Ok(v) => v,
            Err(e) => return Err(Status::unknown(e.to_string())),
        };
    }

    check_is_default_and_delete(&mut config);

    let content = match serde_json::to_string_pretty(&config) {
        Ok(s) => s,
        Err(e) => return Err(Status::unknown(e.to_string())),
    };
    if let Err(e) = file.write_all(content.as_bytes()).await {
        return Err(Status::aborted(e.to_string()));
    };
    Ok(())
}

pub async fn config_read_to_json<P: AsRef<Path>>(
    config_path: P,
) -> std::io::Result<serde_json::Value> {
    let mut file = open_or_create(config_path).await?;

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
async fn open_or_create<P: AsRef<Path>>(config_path: P) -> std::io::Result<tokio::fs::File> {
    let file = match tokio::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(&config_path)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                create_and_reopen(config_path).await?
            } else {
                return Err(e);
            }
        }
    };
    Ok(file)
}

async fn create_and_reopen<P: AsRef<Path>>(config_path: P) -> std::io::Result<tokio::fs::File> {
    let dir = match config_path.as_ref().parent() {
        Some(d) => d,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Parent Directroy Not Found",
            ));
        }
    };
    tokio::fs::create_dir_all(dir).await?;
    let mut f = tokio::fs::File::create(&config_path).await?;
    f.write_all(DEFAULT_CONFIG_FILE_CONTENT.as_bytes()).await?;
    Ok(tokio::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(&config_path)
        .await?)
}
