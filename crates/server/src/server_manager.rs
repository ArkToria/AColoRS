use core_protobuf::acolors_proto::{manager_server::Manager, ShutdownReply, ShutdownRequest};
use spdlog::info;
use tonic::{Request, Response, Status};

pub struct AColoRSManager {
    shutdown_sender: tokio::sync::broadcast::Sender<()>,
}
impl AColoRSManager {
    pub fn new(shutdown_sender: tokio::sync::broadcast::Sender<()>) -> Self {
        Self { shutdown_sender }
    }
}

#[tonic::async_trait]
impl Manager for AColoRSManager {
    async fn shutdown(
        &self,
        request: Request<ShutdownRequest>,
    ) -> Result<Response<ShutdownReply>, Status> {
        info!("Shutdown from {:?}", request.remote_addr());

        self.shutdown_sender
            .send(())
            .map_err(|err| Status::aborted(&format!("Shutdown Error: {}", err)))?;

        let reply = ShutdownReply {};
        Ok(Response::new(reply))
    }
}
