use std::time::Duration;

use core_protobuf::acolors_proto::{TcpingReply, TcpingRequest};
use spdlog::debug;
use tonic::{Request, Response, Status};
use utils::net::tcping;

#[derive(Debug)]
pub struct AColoRSTools;

impl AColoRSTools {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl core_protobuf::acolors_proto::tools_server::Tools for AColoRSTools {
    async fn tcping(
        &self,
        request: Request<TcpingRequest>,
    ) -> Result<Response<TcpingReply>, Status> {
        debug!("Tcping from {:?}", request.remote_addr());

        let target = request.into_inner().target;

        let duration = tcping(target, Duration::from_secs(3))
            .await
            .map(|du| du)
            .map_err(|e| Status::unavailable(e.to_string()))?;

        let duration = duration
            .try_into()
            .map_err(|e| Status::internal(format!("{}", e)))?;

        let reply = TcpingReply {
            duration: Some(duration),
        };
        Ok(Response::new(reply))
    }
}
