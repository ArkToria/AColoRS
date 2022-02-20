use std::sync::Arc;

use acolors_signal::{send_or_error_print, AColorSignal};
use core_protobuf::acolors_proto::{
    core_manager_server::CoreManager, GetIsRunningReply, GetIsRunningRequest, RestartReply,
    RestartRequest, RunReply, RunRequest, SetConfigByNodeIdReply, SetConfigByNodeIdRequest,
    StopReply, StopRequest,
};
use kernel_manager::CoreTool;
use profile_manager::ProfileTaskProducer;
use spdlog::info;
use tokio::sync::{broadcast, Mutex, RwLock};
use tonic::{Request, Response, Status};

type Core = dyn CoreTool + Sync + Send;
pub struct AColoRSCore {
    current_core: Arc<Mutex<Option<Box<Core>>>>,
    profile: Arc<ProfileTaskProducer>,
    inbounds: Arc<RwLock<config_manager::Inbounds>>,
    current_node: Mutex<Option<core_data::NodeData>>,
    signal_sender: broadcast::Sender<profile_manager::AColorSignal>,
}

impl AColoRSCore {
    pub fn new(
        profile: Arc<ProfileTaskProducer>,
        inbounds: Arc<RwLock<config_manager::Inbounds>>,
        signal_sender: broadcast::Sender<profile_manager::AColorSignal>,
    ) -> Self {
        let core = Arc::new(Mutex::new(None));
        Self {
            current_core: core,
            profile,
            inbounds,
            current_node: Mutex::new(None),
            signal_sender,
        }
    }

    pub async fn set_core(&mut self, core: Option<Box<Core>>) {
        let mut core_guard = self.current_core.lock().await;

        *core_guard = core;
    }
}

#[tonic::async_trait]
impl CoreManager for AColoRSCore {
    async fn run(&self, request: Request<RunRequest>) -> Result<Response<RunReply>, Status> {
        info!("Run from {:?}", request.remote_addr());

        let mut core_guard = self.current_core.lock().await;
        let core = &mut *core_guard;
        let core = match core {
            Some(core) => core,
            None => {
                return Err(Status::not_found("Core Not Found"));
            }
        };

        regenerate_config(&self.current_node, &self.inbounds, core).await?;

        if let Err(e) = core.run() {
            return Err(Status::aborted(format!("Core run Error: {}", e)));
        }

        send_or_error_print(&self.signal_sender, AColorSignal::UpdateCoreStatus);

        let reply = RunReply {};
        Ok(Response::new(reply))
    }
    async fn stop(&self, request: Request<StopRequest>) -> Result<Response<StopReply>, Status> {
        info!("Stop from {:?}", request.remote_addr());

        let mut core_guard = self.current_core.lock().await;
        let core = &mut *core_guard;
        let core = match core {
            Some(core) => core,
            None => {
                return Err(Status::not_found("Core Not Found"));
            }
        };

        if let Err(e) = core.stop() {
            return Err(Status::aborted(format!("Core stop Error: {}", e)));
        }

        send_or_error_print(&self.signal_sender, AColorSignal::UpdateCoreStatus);

        let reply = StopReply {};
        Ok(Response::new(reply))
    }

    async fn restart(
        &self,
        request: Request<RestartRequest>,
    ) -> Result<Response<RestartReply>, Status> {
        info!("Restart from {:?}", request.remote_addr());

        let mut core_guard = self.current_core.lock().await;
        let core = &mut *core_guard;
        let core = match core {
            Some(core) => core,
            None => {
                return Err(Status::not_found("Core Not Found"));
            }
        };

        regenerate_config(&self.current_node, &self.inbounds, core).await?;

        if let Err(e) = core.restart() {
            return Err(Status::aborted(format!("Core restart Error: {}", e)));
        }

        send_or_error_print(&self.signal_sender, AColorSignal::UpdateCoreStatus);

        let reply = RestartReply {};
        Ok(Response::new(reply))
    }

    async fn get_is_running(
        &self,
        request: Request<GetIsRunningRequest>,
    ) -> Result<Response<GetIsRunningReply>, Status> {
        info!("Get is_running from {:?}", request.remote_addr());

        let mut core_guard = self.current_core.lock().await;
        let core = &mut *core_guard;
        let core = match core {
            Some(core) => core,
            None => {
                return Err(Status::not_found("Core Not Found"));
            }
        };

        let is_running = core.is_running();

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
            Err(e) => return Err(Status::not_found(format!("Node unavailable: \"{}\"", e))),
        };

        let mut data_guard = self.current_node.lock().await;
        *data_guard = Some(node_data);

        send_or_error_print(&self.signal_sender, AColorSignal::CoreConfigChanged);

        let reply = SetConfigByNodeIdReply {};
        Ok(Response::new(reply))
    }
}

async fn regenerate_config(
    current_node: &Mutex<Option<core_data::NodeData>>,
    inbounds: &Arc<RwLock<config_manager::Inbounds>>,
    core: &mut Box<Core>,
) -> Result<(), Status> {
    let current_node_guard = &*current_node.lock().await;
    let node_data = match current_node_guard {
        Some(d) => d,
        None => {
            return Err(Status::cancelled("No node selected".to_string()));
        }
    };
    let inbounds = &*inbounds.read().await;

    if let Err(e) = core.set_config_by_node_and_inbounds(node_data, inbounds) {
        return Err(Status::cancelled(format!("Core set config Error: {}", e)));
    }
    Ok(())
}
