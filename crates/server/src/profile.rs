use anyhow::Error;
use spdlog::info;

use tonic::{Code, Request, Response, Status};

use crate::protobuf::acolors_proto::{
    profile_manager_server, CountGroupsReply, CountGroupsRequest, GroupList, ListAllGroupsRequest,
};
use profile_manager::{self};

#[derive(Debug)]
pub struct AColoRSProfile {
    manager: profile_manager::ProfileManager,
}
impl AColoRSProfile {
    pub async fn new(path: String) -> Result<AColoRSProfile, Error> {
        let manager = profile_manager::ProfileManager::new(path).await?;
        Ok(AColoRSProfile { manager })
    }
}

#[tonic::async_trait]
impl profile_manager_server::ProfileManager for AColoRSProfile {
    async fn count_groups(
        &self,
        request: Request<CountGroupsRequest>,
    ) -> Result<Response<CountGroupsReply>, Status> {
        info!("Request count groups from {:?}", request.remote_addr());

        let count = match self.manager.count_groups().await {
            Ok(c) => c,
            Err(e) => {
                return Err(Status::new(
                    Code::Unavailable,
                    format!("Count unavailable: \"{}\"", e),
                ))
            }
        };

        let reply = CountGroupsReply {
            count: count as u64,
        };
        Ok(Response::new(reply))
    }

    async fn list_all_groups(
        &self,
        request: Request<ListAllGroupsRequest>,
    ) -> Result<Response<GroupList>, Status> {
        info!("Request list all groups from {:?}", request.remote_addr());

        let group_list: Vec<crate::protobuf::acolors_proto::GroupData> =
            match self.manager.list_all_groups().await {
                Ok(c) => c.into_iter().map(|group| group.into()).collect(),
                Err(e) => {
                    return Err(Status::new(
                        Code::Unavailable,
                        format!("Group unavailable: \"{}\"", e),
                    ))
                }
            };

        let length = group_list.len();
        let reply = GroupList {
            length: length as u64,
            entries: group_list,
        };
        Ok(Response::new(reply))
    }
}
