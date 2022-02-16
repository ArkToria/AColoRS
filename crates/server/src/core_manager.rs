use std::sync::Arc;

use core_protobuf::acolors_proto::{
    core_manager_server::CoreManager, GetIsRunningReply, GetIsRunningRequest, RestartReply,
    RestartRequest, RunReply, RunRequest, SetConfigByNodeIdReply, SetConfigByNodeIdRequest,
    StopReply, StopRequest,
};
use kernel_manager::CoreTool;
use profile_manager::ProfileTaskProducer;
use spdlog::info;
use tokio::sync::{Mutex, RwLock};
use tonic::{Code, Request, Response, Status};

type ConfigType = String;
#[derive(Debug)]
pub struct AColoRSCore<Core>
where
    Core: CoreTool<ConfigType> + Send + Sync + 'static,
{
    core: Arc<Mutex<Core>>,
    profile: Arc<ProfileTaskProducer>,
    inbounds: Arc<RwLock<config_manager::Inbounds>>,
}

impl<Core> AColoRSCore<Core>
where
    Core: CoreTool<ConfigType> + Send + Sync + 'static,
    ConfigType: Send + Sync + 'static,
{
    pub fn new<String>(
        core: Arc<Mutex<Core>>,
        profile: Arc<ProfileTaskProducer>,
        inbounds: Arc<RwLock<config_manager::Inbounds>>,
    ) -> Self {
        Self {
            core,
            profile,
            inbounds,
        }
    }
}

#[tonic::async_trait]
impl<Core> CoreManager for AColoRSCore<Core>
where
    Core: CoreTool<ConfigType> + Send + Sync + 'static,
    ConfigType: Send + Sync + 'static,
{
    async fn run(&self, request: Request<RunRequest>) -> Result<Response<RunReply>, Status> {
        info!("Run from {:?}", request.remote_addr());

        let mut core_guard = self.core.lock().await;
        if let Err(e) = core_guard.run() {
            return Err(Status::new(Code::Aborted, format!("Core run Error: {}", e)));
        }

        let reply = RunReply {};
        Ok(Response::new(reply))
    }
    async fn stop(&self, request: Request<StopRequest>) -> Result<Response<StopReply>, Status> {
        info!("Stop from {:?}", request.remote_addr());

        let mut core_guard = self.core.lock().await;
        if let Err(e) = core_guard.stop() {
            return Err(Status::new(
                Code::Aborted,
                format!("Core stop Error: {}", e),
            ));
        }

        let reply = StopReply {};
        Ok(Response::new(reply))
    }

    async fn restart(
        &self,
        request: Request<RestartRequest>,
    ) -> Result<Response<RestartReply>, Status> {
        info!("restart from {:?}", request.remote_addr());

        let mut core_guard = self.core.lock().await;
        if let Err(e) = core_guard.restart() {
            return Err(Status::new(
                Code::Aborted,
                format!("Core restart Error: {}", e),
            ));
        }

        let reply = RestartReply {};
        Ok(Response::new(reply))
    }

    async fn get_is_running(
        &self,
        request: Request<GetIsRunningRequest>,
    ) -> Result<Response<GetIsRunningReply>, Status> {
        info!("Get is_running from {:?}", request.remote_addr());

        let mut core_guard = self.core.lock().await;
        let is_running = core_guard.is_running();

        let reply = GetIsRunningReply { is_running };
        Ok(Response::new(reply))
    }

    async fn set_config_by_node_id(
        &self,
        request: Request<SetConfigByNodeIdRequest>,
    ) -> Result<Response<SetConfigByNodeIdReply>, Status> {
        info!("get_is_running from {:?}", request.remote_addr());

        let node_id = request.into_inner().node_id;

        let node_data = match self.profile.get_node_by_id(node_id).await {
            Ok(c) => c,
            Err(e) => {
                return Err(Status::new(
                    Code::Unavailable,
                    format!("Node unavailable: \"{}\"", e),
                ))
            }
        };

        let inbounds = &*self.inbounds.read().await;

        let config = match Core::generate_config(&node_data, inbounds) {
            Ok(c) => c,
            Err(e) => {
                return Err(Status::new(
                    Code::Aborted,
                    format!("Generating configuration Error: {}", e),
                ));
            }
        };

        let mut core_guard = self.core.lock().await;
        if let Err(e) = core_guard.set_config(config) {
            return Err(Status::new(
                Code::Aborted,
                format!("Core set config Error: {}", e),
            ));
        }

        let reply = SetConfigByNodeIdReply {};
        Ok(Response::new(reply))
    }
}
