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
use tonic::{Request, Response, Status};

type ConfigType = String;
#[derive(Debug)]
pub struct AColoRSCore<Core>
where
    Core: CoreTool<ConfigType> + Send + Sync + 'static,
{
    core: Arc<Mutex<Core>>,
    profile: Arc<ProfileTaskProducer>,
    inbounds: Arc<RwLock<config_manager::Inbounds>>,
    current_node: Mutex<Option<profile_manager::NodeData>>,
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
            current_node: Mutex::new(None),
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

        regenerate_config(&self.current_node, &self.inbounds, &self.core).await?;

        let mut core_guard = self.core.lock().await;
        if let Err(e) = core_guard.run() {
            return Err(Status::aborted(format!("Core run Error: {}", e)));
        }

        let reply = RunReply {};
        Ok(Response::new(reply))
    }
    async fn stop(&self, request: Request<StopRequest>) -> Result<Response<StopReply>, Status> {
        info!("Stop from {:?}", request.remote_addr());

        let mut core_guard = self.core.lock().await;
        if let Err(e) = core_guard.stop() {
            return Err(Status::aborted(format!("Core stop Error: {}", e)));
        }

        let reply = StopReply {};
        Ok(Response::new(reply))
    }

    async fn restart(
        &self,
        request: Request<RestartRequest>,
    ) -> Result<Response<RestartReply>, Status> {
        info!("Restart from {:?}", request.remote_addr());

        regenerate_config(&self.current_node, &self.inbounds, &self.core).await?;

        let mut core_guard = self.core.lock().await;
        if let Err(e) = core_guard.restart() {
            return Err(Status::aborted(format!("Core restart Error: {}", e)));
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
        info!("Set config by node id from {:?}", request.remote_addr());

        let node_id = request.into_inner().node_id;

        let node_data = match self.profile.get_node_by_id(node_id).await {
            Ok(c) => c,
            Err(e) => return Err(Status::cancelled(format!("Node unavailable: \"{}\"", e))),
        };

        let mut data_guard = self.current_node.lock().await;
        *data_guard = Some(node_data);

        let reply = SetConfigByNodeIdReply {};
        Ok(Response::new(reply))
    }
}

async fn regenerate_config<Core>(
    current_node: &Mutex<Option<profile_manager::NodeData>>,
    inbounds: &Arc<RwLock<config_manager::Inbounds>>,
    core: &Arc<Mutex<Core>>,
) -> Result<(), Status>
where
    Core: CoreTool<ConfigType> + Send + Sync + 'static,
    ConfigType: Send + Sync + 'static,
{
    let current_node_guard = &*current_node.lock().await;
    let node_data = match current_node_guard {
        Some(d) => d,
        None => {
            return Err(Status::cancelled(format!("No node selected")));
        }
    };
    let inbounds = &*inbounds.read().await;

    let config = match Core::generate_config(node_data, inbounds) {
        Ok(c) => c,
        Err(e) => {
            return Err(Status::cancelled(format!(
                "Generating configuration Error: {}",
                e
            )));
        }
    };

    let mut core_guard = core.lock().await;
    if let Err(e) = core_guard.set_config(config) {
        return Err(Status::cancelled(format!("Core set config Error: {}", e)));
    }
    Ok(())
}
