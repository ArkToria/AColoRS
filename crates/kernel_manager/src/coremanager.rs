use std::borrow::BorrowMut;

use crate::{
    v2ray::{configtool::config_to_string, raycore::V2RayCore},
    CoreTool,
};
use anyhow::anyhow;
use core_protobuf::v2ray_proto::*;
use core_protobuf::v2ray_proto::{
    inbound_object::InboundSettings, policy_object::SystemPolicyObject, routing_object::RuleObject,
    ApiObject, InboundObject, PolicyObject, RoutingObject, StatsObject, V2RayConfig,
};

type Core = dyn CoreTool + Sync + Send;
pub struct ExternalCore {
    pub tag: String,
    pub core: Box<Core>,
}

pub struct APIConfig {
    pub port: u32,
    pub listen: String,
}
pub struct RayCore {
    ray_core: Option<V2RayCore>,
    external_cores: Vec<ExternalCore>,
    config: V2RayConfig,
    api: Option<APIConfig>,
}

impl Default for RayCore {
    fn default() -> Self {
        Self::new()
    }
}
impl RayCore {
    pub fn new() -> Self {
        Self {
            api: None,
            ray_core: None,
            external_cores: Vec::new(),
            config: V2RayConfig::default(),
        }
    }
    pub fn set_ray_core(&mut self, core: V2RayCore) {
        self.ray_core = Some(core);
    }
    pub fn external_cores_mut(&mut self) -> &mut Vec<ExternalCore> {
        &mut self.external_cores
    }
    pub fn set_config_by_node_and_inbounds(
        &mut self,
        node_data: &core_data::NodeData,
        inbounds: &config_manager::Inbounds,
    ) -> anyhow::Result<()> {
        self.config = crate::v2ray::configtool::generate_config(node_data, inbounds)?;
        self.set_api_object();
        Ok(())
    }
    pub fn set_stat(&mut self, enable: bool) {
        let config = self.config.borrow_mut();
        if enable && config.stats.is_none() {
            config.stats = Some(StatsObject {});
        } else if (!enable) && config.stats.is_some() {
            config.stats = None;
        }
    }
    pub fn api_mut(&mut self) -> &mut Option<ApiObject> {
        self.config.api.borrow_mut()
    }
    pub fn routing_mut(&mut self) -> &mut Option<RoutingObject> {
        self.config.routing.borrow_mut()
    }
    pub fn inbound_mut(&mut self) -> &mut Vec<InboundObject> {
        self.config.inbounds.borrow_mut()
    }
    pub fn policy_mut(&mut self) -> &mut Option<PolicyObject> {
        self.config.policy.borrow_mut()
    }
    pub fn set_api_address(&mut self, listen: &str, port: u32) {
        self.api = Some(APIConfig {
            port,
            listen: listen.to_string(),
        })
    }
    pub fn api_config(&self) -> &Option<APIConfig> {
        &self.api
    }
    fn set_api_object(&mut self) {
        match &self.api {
            Some(api) => {
                let mut inbound = InboundObject {
                    listen: api.listen.to_string(),
                    port: api.port,
                    protocol: "dokodemo-door".to_string(),
                    tag: "ACROSS_API_INBOUND".to_string(),
                    ..Default::default()
                };
                let doko_setting = dokodemo_door_object::InboundConfigurationObject {
                    address: "127.0.0.1".to_string(),
                    ..Default::default()
                };
                inbound.settings = Some(InboundSettings {
                    kind: Some(inbound_object::inbound_settings::Kind::DokodemoDoor(
                        doko_setting,
                    )),
                });
                self.inbound_mut().push(inbound);
                self.inbound_mut().rotate_right(1);

                self.set_stat(true);
                *self.api_mut() = Some(ApiObject {
                    tag: "ACROSS_API".to_string(),
                    services: vec!["LoggerService".to_string(), "StatsService".to_string()],
                });
                let routing = RoutingObject {
                    rules: vec![RuleObject {
                        r#type: "field".to_string(),
                        outbound_tag: "ACROSS_API".to_string(),
                        inbound_tag: vec!["ACROSS_API_INBOUND".to_string()],
                        ..Default::default()
                    }],
                    ..Default::default()
                };
                *self.routing_mut() = Some(routing);

                let policy = PolicyObject {
                    system: Some(SystemPolicyObject {
                        stats_inbound_downlink: true,
                        stats_inbound_uplink: true,
                        stats_outbound_downlink: true,
                        stats_outbound_uplink: true,
                    }),
                    ..Default::default()
                };
                *self.policy_mut() = Some(policy);
            }
            None => {}
        }
    }
}

impl CoreTool for RayCore {
    fn run(&mut self) -> anyhow::Result<()> {
        let run_result = self
            .external_cores
            .iter_mut()
            .map(|external_core| external_core.core.run().ok().map(|_| external_core));
        let abort_result: Vec<_> = run_result
            .flatten()
            .map(|external_core| external_core.core.stop())
            .filter_map(|result| result.err())
            .collect();
        if !abort_result.is_empty() {
            let error = abort_result
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<String>>()
                .join("\n");
            return Err(anyhow!(error));
        }
        match self.ray_core.as_mut() {
            Some(ray_core) => {
                ray_core.set_config(config_to_string(&self.config)?)?;
                ray_core.run()
            }
            None => Err(anyhow!("RayCore Not Found.")),
        }
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        let stop_result: Vec<_> = self
            .external_cores
            .iter_mut()
            .filter_map(|external_core| external_core.core.stop().err())
            .collect();
        if !stop_result.is_empty() {
            let error = stop_result
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<String>>()
                .join("\n");
            return Err(anyhow!(error));
        }
        match self.ray_core.as_mut() {
            Some(ray_core) => ray_core.stop(),
            None => Err(anyhow!("RayCore Not Found.")),
        }
    }

    fn is_running(&mut self) -> bool {
        self.ray_core
            .as_mut()
            .map(|core| core.is_running())
            .unwrap_or(false)
    }

    fn set_config(&mut self, config: String) -> anyhow::Result<()> {
        self.ray_core
            .as_mut()
            .map(|core| core.set_config(config))
            .unwrap_or_else(|| Err(anyhow!("RayCore Not Found.")))
    }

    fn get_stdout(&mut self) -> Option<std::process::ChildStdout> {
        self.ray_core
            .as_mut()
            .map(|core| core.get_stdout())
            .unwrap_or(None)
    }

    fn get_version(&self) -> &str {
        self.ray_core
            .as_ref()
            .map(|core| core.get_version())
            .unwrap_or("")
    }

    fn get_name(&self) -> &str {
        self.ray_core
            .as_ref()
            .map(|core| core.get_name())
            .unwrap_or("")
    }

    fn get_config(&self) -> &str {
        self.ray_core
            .as_ref()
            .map(|core| core.get_config())
            .unwrap_or("")
    }

    fn set_config_by_node_and_socks_inbound(
        &mut self,
        node_data: &core_data::NodeData,
        inbound: &config_manager::SOCKS5Inbound,
    ) -> anyhow::Result<()> {
        match self.ray_core.as_mut() {
            Some(ray_core) => {
                let config =
                    crate::v2ray::configtool::generate_config_by_socks(node_data, inbound)?;
                ray_core.set_config(config)?;
                Ok(())
            }
            None => Err(anyhow!("RayCore Not Found.")),
        }
    }
}
