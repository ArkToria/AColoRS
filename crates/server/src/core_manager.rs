use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    sync::Arc,
};

use acolors_signal::{send_or_warn_print, AColorSignal};
use anyhow::{anyhow, Result};
use core_protobuf::acolors_proto::{
    core_manager_server::CoreManager, GetCoreTagReply, GetCoreTagRequest, GetCurrentNodeRequest,
    GetIsRunningReply, GetIsRunningRequest, NodeData, RestartReply, RestartRequest, RunReply,
    RunRequest, SetConfigByNodeIdReply, SetConfigByNodeIdRequest, SetCoreByTagReply,
    SetCoreByTagRequest, SetDefaultConfigByNodeIdReply, SetDefaultConfigByNodeIdRequest, StopReply,
    StopRequest,
};
use kernel_manager::{create_core_by_path, CoreTool};
use profile_manager::ProfileTaskProducer;
use spdlog::{error, info};
use tokio::sync::{broadcast, Mutex, RwLock};
use tonic::{Request, Response, Status};

type Core = dyn CoreTool + Sync + Send;

type CurrentCore = Mutex<Option<Box<Core>>>;
type InboundsLock = Arc<RwLock<config_manager::Inbounds>>;
type CurrentNodeLock = Mutex<Option<core_data::NodeData>>;
type CoreTag = Mutex<String>;
pub struct AColoRSCore {
    core_tag: CoreTag,
    current_core: CurrentCore,
    profile: Arc<ProfileTaskProducer>,
    inbounds: InboundsLock,
    current_node: CurrentNodeLock,
    signal_sender: broadcast::Sender<profile_manager::AColorSignal>,
    core_map: HashMap<String, (String, OsString)>,
}

impl AColoRSCore {
    pub async fn create(
        profile: Arc<ProfileTaskProducer>,
        inbounds: InboundsLock,
        signal_sender: broadcast::Sender<profile_manager::AColorSignal>,
    ) -> Self {
        let core = Mutex::new(None);
        let core_map = HashMap::new();
        let current_node = Mutex::new(None);
        let current_node_id = profile.get_runtime_value("CURRENT_NODE_ID").await.ok();
        let default_node_id = profile.get_runtime_value("DEFAULT_NODE_ID").await.ok();

        let mut receiver = signal_sender.subscribe();
        let node_selected = Self::check_and_set_config(default_node_id, &profile, &current_node)
            .await
            || Self::check_and_set_config(current_node_id, &profile, &current_node).await;
        let signal = receiver.try_recv().ok();
        if node_selected & signal.is_some() {
            info!("Default Node Selected");
        }

        Self {
            current_core: core,
            profile,
            inbounds,
            current_node,
            signal_sender,
            core_map,
            core_tag: Mutex::new(String::new()),
        }
    }

    pub async fn get_current_node(&self) -> Result<NodeData, Status> {
        self.current_node
            .lock()
            .await
            .clone()
            .ok_or_else(|| Status::not_found("Node not found"))
            .map(|node| node.into())
    }

    pub async fn restart(&self) -> Result<(), Status> {
        let mut core_guard = self.current_core.lock().await;
        let core = &mut *core_guard;
        let core = core
            .as_mut()
            .ok_or_else(|| Status::not_found("Core Not Found"))?;

        regenerate_config(&self.current_node, &self.inbounds, core).await?;

        core.restart()
            .map_err(|e| Status::aborted(format!("Core restart Error: {}", e)))?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Status> {
        let mut core_guard = self.current_core.lock().await;
        let core = &mut *core_guard;
        let core = core
            .as_mut()
            .ok_or_else(|| Status::not_found("Core Not Found"))?;

        core.stop()
            .map_err(|e| Status::aborted(format!("Core stop Error: {}", e)))?;
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Status> {
        let mut core_guard = self.current_core.lock().await;
        let core = &mut *core_guard;
        let core = core
            .as_mut()
            .ok_or_else(|| Status::not_found("Core Not Found"))?;

        regenerate_config(&self.current_node, &self.inbounds, core).await?;

        core.run()
            .map_err(|e| Status::aborted(format!("Core run Error: {}", e)))?;

        Ok(())
    }
    pub async fn set_config(
        profile: &Arc<ProfileTaskProducer>,
        current_node: &CurrentNodeLock,
        node_id: i32,
    ) -> Result<(), Status> {
        let node_data = profile
            .get_node_by_id(node_id)
            .await
            .map_err(|e| Status::not_found(format!("Node unavailable: \"{}\"", e)))?;

        {
            let mut data_guard = current_node.lock().await;
            *data_guard = Some(node_data);
        }

        profile
            .set_runtime_value("CURRENT_NODE_ID", node_id.to_string())
            .await
            .unwrap_or_else(|e| error!("Set Config Error: {}", e));

        Ok(())
    }
    async fn check_and_set_config(
        node_id_string: Option<String>,
        profile: &Arc<ProfileTaskProducer>,
        current_node: &CurrentNodeLock,
    ) -> bool {
        if let Some(node_id_string) = node_id_string {
            match node_id_string.parse() {
                Ok(node_id) => {
                    let result = Self::set_config(profile, current_node, node_id).await.ok();
                    return result.is_some();
                }
                Err(e) => {
                    error!("Current Node Id Parsing Error:{}", e);
                }
            }
        }
        false
    }

    pub async fn set_core(&self, core_tag: &str) -> Result<()> {
        self.set_core_tag(core_tag).await;
        let mut core_guard = self.current_core.lock().await;

        let core = &mut *core_guard;
        if let Some(core) = core {
            if core.is_running() {
                core.stop()?
            }
        }

        let (core_name, path) = self
            .core_map
            .get(core_tag)
            .ok_or_else(|| anyhow!("Core Not Added"))?;

        let core = create_core_by_path(path, core_name).map_err(|e| {
            error!("Core not found : {}", e);
            e
        })?;

        info!(
            "Core <{}> version ({})",
            core.get_name(),
            core.get_version()
        );

        *core_guard = Some(core);
        Ok(())
    }

    async fn set_core_tag(&self, core_tag: &str) {
        let mut tag_guard = self.core_tag.lock().await;
        *tag_guard = core_tag.to_string();
    }
    pub async fn add_core<S: AsRef<OsStr>>(
        &mut self,
        core_name: &str,
        tag: &str,
        core_path: S,
    ) -> Result<()> {
        let tag = tag.to_string();
        let core_path = core_path.as_ref().to_os_string();
        let core_name = core_name.to_string();
        self.core_map.insert(tag, (core_name, core_path));
        Ok(())
    }
}

#[tonic::async_trait]
impl CoreManager for AColoRSCore {
    async fn get_current_node(
        &self,
        request: Request<GetCurrentNodeRequest>,
    ) -> Result<Response<NodeData>, Status> {
        info!("Get current node from {:?}", request.remote_addr());
        Ok(Response::new(self.get_current_node().await?))
    }
    async fn run(&self, request: Request<RunRequest>) -> Result<Response<RunReply>, Status> {
        info!("Run from {:?}", request.remote_addr());

        self.run().await?;

        send_or_warn_print(&self.signal_sender, AColorSignal::UpdateCoreStatus);

        let reply = RunReply {};
        Ok(Response::new(reply))
    }
    async fn stop(&self, request: Request<StopRequest>) -> Result<Response<StopReply>, Status> {
        info!("Stop from {:?}", request.remote_addr());

        self.stop().await?;

        send_or_warn_print(&self.signal_sender, AColorSignal::UpdateCoreStatus);

        let reply = StopReply {};
        Ok(Response::new(reply))
    }

    async fn restart(
        &self,
        request: Request<RestartRequest>,
    ) -> Result<Response<RestartReply>, Status> {
        info!("Restart from {:?}", request.remote_addr());

        self.restart().await?;

        send_or_warn_print(&self.signal_sender, AColorSignal::UpdateCoreStatus);

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
        let core = core
            .as_mut()
            .ok_or_else(|| Status::not_found("Core Not Found"))?;

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

        Self::set_config(&self.profile, &self.current_node, node_id).await?;

        send_or_warn_print(&self.signal_sender, AColorSignal::CoreConfigChanged);

        let reply = SetConfigByNodeIdReply {};
        Ok(Response::new(reply))
    }

    async fn set_default_config_by_node_id(
        &self,
        request: Request<SetDefaultConfigByNodeIdRequest>,
    ) -> Result<Response<SetDefaultConfigByNodeIdReply>, Status> {
        info!(
            "Set default config by node id from {:?}",
            request.remote_addr()
        );

        let node_id = request.into_inner().node_id;

        self.profile
            .set_runtime_value("DEFAULT_NODE_ID", node_id.to_string())
            .await
            .unwrap_or_else(|e| error!("Set Config Error: {}", e));

        let reply = SetDefaultConfigByNodeIdReply {};
        Ok(Response::new(reply))
    }

    async fn set_core_by_tag(
        &self,
        request: Request<SetCoreByTagRequest>,
    ) -> Result<Response<SetCoreByTagReply>, Status> {
        info!("Set core by tag id from {:?}", request.remote_addr());

        let tag = request.into_inner().tag;

        self.set_core(&tag)
            .await
            .map_err(|e| Status::not_found(format!("Core not found: \"{}\"", e)))?;

        send_or_warn_print(&self.signal_sender, AColorSignal::CoreChanged);

        let reply = SetCoreByTagReply {};
        Ok(Response::new(reply))
    }
    async fn get_core_tag(
        &self,
        request: Request<GetCoreTagRequest>,
    ) -> Result<Response<GetCoreTagReply>, Status> {
        info!("Get core tag from {:?}", request.remote_addr());

        let tag = self.core_tag.lock().await.clone();
        let reply = GetCoreTagReply { tag };
        Ok(Response::new(reply))
    }
}

async fn regenerate_config(
    current_node: &CurrentNodeLock,
    inbounds: &InboundsLock,
    core: &mut Box<Core>,
) -> Result<(), Status> {
    let current_node_guard = &*current_node.lock().await;
    let node_data = current_node_guard
        .as_ref()
        .ok_or_else(|| Status::cancelled("No node selected"))?;

    let inbounds = &*inbounds.read().await;

    core.set_config_by_node_and_inbounds(node_data, inbounds)
        .map_err(|e| Status::cancelled(format!("Core set config Error: {}", e)))
}
