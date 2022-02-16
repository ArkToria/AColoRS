use std::sync::Arc;

use core_protobuf::acolors_proto::{config_manager_server::ConfigManager, *};
use spdlog::info;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct AColoRSConfig {
    inbounds: Arc<RwLock<config_manager::Inbounds>>,
}

impl AColoRSConfig {
    pub async fn new(inbounds: Arc<RwLock<config_manager::Inbounds>>) -> Self {
        Self { inbounds }
    }
}

#[tonic::async_trait]
impl ConfigManager for AColoRSConfig {
    async fn set_inbounds(
        &self,
        request: Request<Inbounds>,
    ) -> Result<Response<SetInboundsReply>, Status> {
        info!("Set inbounds from {:?}", request.remote_addr());
        let mut inbounds_write = self.inbounds.write().await;

        let inbounds = request.into_inner();

        *inbounds_write = inbounds.into();

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
