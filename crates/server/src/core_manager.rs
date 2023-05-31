use std::{
    ffi::OsStr,
    sync::{atomic::AtomicBool, Arc},
};

use acolors_signal::{send_or_warn_print, AColorSignal};
use anyhow::{anyhow, Result};
use core_protobuf::acolors_proto::{
    core_manager_server::CoreManager, GetCoreInfoReply, GetCoreInfoRequest, GetCoreTagReply,
    GetCoreTagRequest, GetCurrentNodeRequest, GetIsRunningReply, GetIsRunningRequest,
    GetTrafficInfoRequest, ListAllTagsReply, ListAllTagsRequest, NodeData, RestartReply,
    RestartRequest, RunReply, RunRequest, SetApiStatusReply, SetApiStatusRequest,
    SetConfigByNodeIdReply, SetConfigByNodeIdRequest, SetCoreByTagReply, SetCoreByTagRequest,
    SetDefaultConfigByNodeIdReply, SetDefaultConfigByNodeIdRequest, StopReply, StopRequest,
    TrafficInfo,
};
use kernel_manager::{
    coremanager::{APIConfig, RayCore},
    create_core_by_path,
    v2ray::raycore::V2RayCore,
    CoreTool,
};
use profile_manager::Profile;
use spdlog::{debug, error, info, warn};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::{broadcast, Mutex, RwLock},
};
use tonic::{Request, Response, Status};
use utils::net::tcp_get_available_port;

use crate::traffic_speed::TrafficInfoUpdater;

type CurrentCore = Mutex<RayCore>;
type InboundsLock = Arc<RwLock<config_manager::Inbounds>>;
type CurrentNodeLock = Mutex<Option<core_data::NodeData>>;
type CoreTag = Mutex<String>;

pub struct AcolorsCore {
    core_tag: CoreTag,
    current_core: Arc<CurrentCore>,
    profile: Arc<Profile>,
    inbounds: InboundsLock,
    current_node: CurrentNodeLock,
    signal_sender: broadcast::Sender<profile_manager::AColorSignal>,
    traffic_info: Arc<Mutex<TrafficInfo>>,
    speed_updater: Arc<Mutex<TrafficInfoUpdater>>,
    enable_api: Arc<AtomicBool>,
}

impl AcolorsCore {
    pub async fn create(
        profile: Arc<Profile>,
        inbounds: InboundsLock,
        signal_sender: broadcast::Sender<profile_manager::AColorSignal>,
    ) -> Self {
        let core = Mutex::new(RayCore::new());
        let current_node = Mutex::new(None);
        let current_node_id = profile.runtime_value.get_by_key("CURRENT_NODE_ID").await;
        let default_node_id = profile.runtime_value.get_by_key("DEFAULT_NODE_ID").await;
        let traffic_info = Arc::new(Mutex::new(TrafficInfo::default()));

        let mut receiver = signal_sender.subscribe();
        let node_selected =
            Self::check_and_set_config(default_node_id, &profile, &current_node, &signal_sender)
                .await
                || Self::check_and_set_config(
                    current_node_id,
                    &profile,
                    &current_node,
                    &signal_sender,
                )
                .await;
        let signal = receiver.try_recv().ok();
        if node_selected & signal.is_some() {
            info!("Default Node Selected");
        }

        Self {
            current_core: Arc::new(core),
            profile,
            inbounds,
            current_node,
            signal_sender,
            core_tag: Mutex::new(String::new()),
            speed_updater: Arc::new(Mutex::new(TrafficInfoUpdater::new(traffic_info.clone()))),
            enable_api: Arc::new(AtomicBool::from(false)),
            traffic_info,
        }
    }

    pub async fn get_current_node(&self) -> Result<NodeData, Status> {
        self.current_node
            .lock()
            .await
            .clone()
            .ok_or_else(|| Status::not_found("Node not found"))
    }

    pub async fn restart(&self) -> Result<(), Status> {
        let mut core_guard = self.current_core.lock().await;
        let core = &mut *core_guard;

        regenerate_config(&self.current_node, &self.inbounds, core).await?;

        core.restart()
            .map_err(|e| Status::aborted(format!("Core restart Error: {}", e)))?;

        tokio::spawn(Self::update_updater(
            self.current_core.clone(),
            self.enable_api.clone(),
            self.speed_updater.clone(),
            core_guard.api_ref().clone(),
        ));

        Ok(())
    }
    pub async fn update_updater(
        current_core: Arc<CurrentCore>,
        enable_api: Arc<AtomicBool>,
        speed_updater: Arc<Mutex<TrafficInfoUpdater>>,
        api: Option<APIConfig>,
    ) {
        let is_running = current_core.lock().await.get_is_running();
        if is_running {
            if enable_api.load(std::sync::atomic::Ordering::SeqCst) {
                let mut updater_guard = speed_updater.lock().await;
                if let Some(api) = api {
                    if let Err(e) = updater_guard
                        .start(format!("http://{}:{}", api.listen, api.port), "PROXY")
                        .await
                    {
                        error!("Traffic Updater Start Error: {}", e);
                    }
                }
            }
        } else {
            let mut updater_guard = speed_updater.lock().await;
            if let Err(e) = updater_guard.stop().await {
                warn!("Stop Traffic Updater Error: {}", e);
            }
        }
    }

    pub async fn stop(&self) -> Result<(), Status> {
        let mut core_guard = self.current_core.lock().await;
        let core = &mut *core_guard;

        core.stop()
            .map_err(|e| Status::aborted(format!("Core stop Error: {}", e)))?;

        tokio::spawn(Self::update_updater(
            self.current_core.clone(),
            self.enable_api.clone(),
            self.speed_updater.clone(),
            core_guard.api_ref().clone(),
        ));

        Ok(())
    }

    pub async fn run(&self) -> Result<(), Status> {
        let mut core_guard = self.current_core.lock().await;

        let core = &mut *core_guard;

        regenerate_config(&self.current_node, &self.inbounds, core).await?;

        core.run()
            .map_err(|e| Status::aborted(format!("Core run Error: {}", e)))?;

        if let Some(stdout) = core.get_stdout() {
            print_stdout(stdout).await;
        }

        tokio::spawn(Self::update_updater(
            self.current_core.clone(),
            self.enable_api.clone(),
            self.speed_updater.clone(),
            core_guard.api_ref().clone(),
        ));

        Ok(())
    }
    pub async fn set_config(
        profile: &Arc<Profile>,
        current_node: &CurrentNodeLock,
        node_id: i64,
        signal_sender: &broadcast::Sender<AColorSignal>,
    ) -> Result<(), Status> {
        let node_data = profile
            .group_list
            .default_group()
            .query(node_id)
            .await
            .map_err(|e| Status::not_found(format!("Node unavailable: \"{}\"", e)))?
            .to_data();

        {
            let mut data_guard = current_node.lock().await;
            *data_guard = Some(node_data);
        }
        send_or_warn_print(signal_sender, AColorSignal::CoreConfigChanged);

        profile
            .runtime_value
            .set_by_key("CURRENT_NODE_ID", node_id.to_string())
            .await
            .unwrap_or_else(|e| error!("Set Config Error: {}", e));
        send_or_warn_print(
            signal_sender,
            AColorSignal::RuntimeValueChanged {
                key: "CURRENT_NODE_ID".to_string(),
            },
        );

        Ok(())
    }
    async fn check_and_set_config(
        node_id_string: Option<String>,
        profile: &Arc<Profile>,
        current_node: &CurrentNodeLock,
        signal_sender: &broadcast::Sender<AColorSignal>,
    ) -> bool {
        if let Some(node_id_string) = node_id_string {
            match node_id_string.parse() {
                Ok(node_id) => {
                    let result = Self::set_config(profile, current_node, node_id, signal_sender)
                        .await
                        .ok();
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

        let core_manager = &mut *core_guard;
        if core_manager.get_is_running() {
            core_manager.stop()?
        }

        let core = core_manager
            .external_cores_mut()
            .get(core_tag)
            .ok_or_else(|| anyhow!("Core Not Added"))?;

        print_core_name_version(core.as_ref());

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

        match core_name.to_ascii_lowercase().as_str() {
            "v2ray" => {
                let core = match V2RayCore::new(&core_path) {
                    Ok(c) => c,
                    Err(e) => {
                        error!(
                            "{} Core (Path:{:?}) not found : {}",
                            &core_name, &core_path, e
                        );
                        return Ok(());
                    }
                };

                print_core_name_version(&core);

                let mut core_guard = self.current_core.lock().await;

                let core_manager = &mut *core_guard;
                core_manager.set_ray_core(core);
            }
            _ => {
                let core = match create_core_by_path(&core_path, &core_name) {
                    Ok(c) => c,
                    Err(e) => {
                        error!(
                            "{} Core (Path:{:?}) not found : {}",
                            &core_name, &core_path, e
                        );
                        return Ok(());
                    }
                };

                print_core_name_version(core.as_ref());

                let mut core_guard = self.current_core.lock().await;

                let core_manager = &mut *core_guard;
                core_manager.external_cores_mut().insert(tag, core);
            }
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl CoreManager for AcolorsCore {
    async fn get_current_node(
        &self,
        request: Request<GetCurrentNodeRequest>,
    ) -> Result<Response<NodeData>, Status> {
        debug!("Get current node from {:?}", request.remote_addr());
        Ok(Response::new(self.get_current_node().await?))
    }
    async fn run(&self, request: Request<RunRequest>) -> Result<Response<RunReply>, Status> {
        debug!("Run from {:?}", request.remote_addr());

        self.run().await?;

        send_or_warn_print(&self.signal_sender, AColorSignal::UpdateCoreStatus);

        let reply = RunReply {};
        Ok(Response::new(reply))
    }
    async fn stop(&self, request: Request<StopRequest>) -> Result<Response<StopReply>, Status> {
        debug!("Stop from {:?}", request.remote_addr());

        self.stop().await?;

        send_or_warn_print(&self.signal_sender, AColorSignal::UpdateCoreStatus);

        let reply = StopReply {};
        Ok(Response::new(reply))
    }

    async fn restart(
        &self,
        request: Request<RestartRequest>,
    ) -> Result<Response<RestartReply>, Status> {
        debug!("Restart from {:?}", request.remote_addr());

        self.restart().await?;

        send_or_warn_print(&self.signal_sender, AColorSignal::UpdateCoreStatus);

        let reply = RestartReply {};
        Ok(Response::new(reply))
    }

    async fn get_is_running(
        &self,
        request: Request<GetIsRunningRequest>,
    ) -> Result<Response<GetIsRunningReply>, Status> {
        debug!("Get is_running from {:?}", request.remote_addr());

        let mut core_guard = self.current_core.lock().await;
        let core = &mut *core_guard;

        let is_running = core.get_is_running();

        let reply = GetIsRunningReply { is_running };
        Ok(Response::new(reply))
    }

    async fn set_config_by_node_id(
        &self,
        request: Request<SetConfigByNodeIdRequest>,
    ) -> Result<Response<SetConfigByNodeIdReply>, Status> {
        debug!("Set config by node id from {:?}", request.remote_addr());

        let node_id = request.into_inner().node_id;

        Self::set_config(
            &self.profile,
            &self.current_node,
            node_id as i64,
            &self.signal_sender,
        )
        .await?;

        send_or_warn_print(&self.signal_sender, AColorSignal::CoreConfigChanged);

        let reply = SetConfigByNodeIdReply {};
        Ok(Response::new(reply))
    }

    async fn set_default_config_by_node_id(
        &self,
        request: Request<SetDefaultConfigByNodeIdRequest>,
    ) -> Result<Response<SetDefaultConfigByNodeIdReply>, Status> {
        debug!(
            "Set default config by node id from {:?}",
            request.remote_addr()
        );

        let node_id = request.into_inner().node_id;

        self.profile
            .runtime_value
            .set_by_key("DEFAULT_NODE_ID", node_id.to_string())
            .await
            .map_err(|e| Status::cancelled(format!("Set Config Error: {}", e)))?;
        send_or_warn_print(
            &self.signal_sender,
            AColorSignal::RuntimeValueChanged {
                key: "DEFAULT_NODE_ID".to_string(),
            },
        );

        let reply = SetDefaultConfigByNodeIdReply {};
        Ok(Response::new(reply))
    }

    async fn set_core_by_tag(
        &self,
        request: Request<SetCoreByTagRequest>,
    ) -> Result<Response<SetCoreByTagReply>, Status> {
        debug!("Set core by tag id from {:?}", request.remote_addr());

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
        debug!("Get core tag from {:?}", request.remote_addr());

        let tag = self.core_tag.lock().await.clone();
        let reply = GetCoreTagReply { tag };
        Ok(Response::new(reply))
    }
    async fn get_core_info(
        &self,
        request: Request<GetCoreInfoRequest>,
    ) -> Result<Response<GetCoreInfoReply>, Status> {
        debug!("Get core info from {:?}", request.remote_addr());

        let (name, version) = {
            let core = &*self.current_core.lock().await;
            (core.get_name().to_string(), core.get_version().to_string())
        };

        let reply = GetCoreInfoReply { name, version };
        Ok(Response::new(reply))
    }
    async fn list_all_tags(
        &self,
        request: Request<ListAllTagsRequest>,
    ) -> Result<Response<ListAllTagsReply>, Status> {
        debug!("List all tags from {:?}", request.remote_addr());

        let tags;
        {
            tags = self
                .current_core
                .lock()
                .await
                .external_cores_mut()
                .keys()
                .into_iter()
                .cloned()
                .collect();
        }

        let reply = ListAllTagsReply { tags };
        Ok(Response::new(reply))
    }
    async fn set_api_status(
        &self,
        request: Request<SetApiStatusRequest>,
    ) -> Result<Response<SetApiStatusReply>, Status> {
        debug!("Set api status from {:?}", request.remote_addr());

        let enable_api = request.into_inner().enable;

        self.enable_api
            .store(enable_api, std::sync::atomic::Ordering::SeqCst);
        if enable_api {
            let mut core_guard = self.current_core.lock().await;
            let port = tcp_get_available_port(11451..19198).unwrap_or(19200);
            info!("api port: {}", port);
            core_guard.set_api_address("127.0.0.1", port as u32);
        } else {
            let mut core_guard = self.current_core.lock().await;
            core_guard.set_api_address("", 0);
        }

        send_or_warn_print(&self.signal_sender, AColorSignal::SetApiStatus);

        let reply = SetApiStatusReply {};
        Ok(Response::new(reply))
    }
    async fn get_traffic_info(
        &self,
        request: Request<GetTrafficInfoRequest>,
    ) -> Result<Response<TrafficInfo>, Status> {
        debug!("Get traffic speed from {:?}", request.remote_addr());

        let reply = self.traffic_info.lock().await.clone();
        Ok(Response::new(reply))
    }
}

async fn regenerate_config(
    current_node: &CurrentNodeLock,
    inbounds: &InboundsLock,
    core: &mut RayCore,
) -> Result<(), Status> {
    let current_node_guard = &*current_node.lock().await;
    let node_data = current_node_guard
        .as_ref()
        .ok_or_else(|| Status::cancelled("No node selected"))?;

    let inbounds = &*inbounds.read().await;

    core.set_config_by_node_and_inbounds(node_data, inbounds)
        .map_err(|e| Status::cancelled(format!("Core set config Error: {}", e)))
}

async fn print_stdout(stdout: std::process::ChildStdout) {
    let stdout = tokio::process::ChildStdout::from_std(stdout);
    match stdout {
        Ok(stdout) => {
            tokio::spawn(async move {
                let mut buf = String::new();
                let mut buf_reader = BufReader::new(stdout);
                while let Ok(count) = buf_reader.read_line(&mut buf).await {
                    if count == 0 {
                        break;
                    }

                    info!("{}", buf.trim_end());
                    buf.clear();
                }
            });
        }
        Err(e) => error!("Process output error: {}", e),
    }
}

fn print_core_name_version(core: &dyn CoreTool) {
    info!(
        "Core <{}> version ({})",
        core.get_name(),
        core.get_version()
    );
}
