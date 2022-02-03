use spdlog::info;

use tonic::{Request, Response, Status};

use crate::protobuf::acolors_proto::{
    profile_manager_server::ProfileManager, CountGroupsReply, CountGroupsRequest,
};

#[derive(Default)]
pub struct AColoRSProfile {}
impl AColoRSProfile {}

#[tonic::async_trait]
impl ProfileManager for AColoRSProfile {
    async fn count_groups(
        &self,
        request: Request<CountGroupsRequest>,
    ) -> Result<Response<CountGroupsReply>, Status> {
        info!("Request count groups from {:?}", request.remote_addr());

        let reply = CountGroupsReply { count: 1 };
        Ok(Response::new(reply))
    }
}
