use anyhow::Error;
use spdlog::info;

use tonic::{Code, Request, Response, Status};

use crate::protobuf::acolors_proto::*;
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

    async fn count_nodes(
        &self,
        request: Request<CountNodesRequest>,
    ) -> Result<Response<CountNodesReply>, Status> {
        info!("Request count nodes from {:?}", request.remote_addr());

        let group_id = request.into_inner().group_id;

        let count = match self.manager.count_nodes(group_id).await {
            Ok(c) => c,
            Err(e) => {
                return Err(Status::new(
                    Code::Unavailable,
                    format!("Count unavailable: \"{}\"", e),
                ))
            }
        };

        let reply = CountNodesReply {
            count: count as u64,
        };
        Ok(Response::new(reply))
    }

    async fn list_all_nodes(
        &self,
        request: Request<ListAllNodesRequest>,
    ) -> Result<Response<NodeList>, Status> {
        info!("Request list all nodes from {:?}", request.remote_addr());

        let group_id = request.into_inner().group_id;

        let group_list: Vec<crate::protobuf::acolors_proto::NodeData> =
            match self.manager.list_all_nodes(group_id).await {
                Ok(c) => c.into_iter().map(|group| group.into()).collect(),
                Err(e) => {
                    return Err(Status::new(
                        Code::Unavailable,
                        format!("Node unavailable: \"{}\"", e),
                    ))
                }
            };

        let length = group_list.len();
        let reply = NodeList {
            length: length as u64,
            entries: group_list,
        };
        Ok(Response::new(reply))
    }

    async fn get_group_by_id(
        &self,
        request: Request<GetGroupByIdRequest>,
    ) -> Result<Response<GroupData>, Status> {
        info!("Request get group by Id from {:?}", request.remote_addr());

        let group_id = request.into_inner().group_id;

        let group_data: crate::protobuf::acolors_proto::GroupData =
            match self.manager.get_group_by_id(group_id).await {
                Ok(c) => c.into(),
                Err(e) => {
                    return Err(Status::new(
                        Code::Unavailable,
                        format!("Group unavailable: \"{}\"", e),
                    ))
                }
            };

        let reply = group_data;

        Ok(Response::new(reply))
    }
    async fn get_node_by_id(
        &self,
        request: Request<GetNodeByIdRequest>,
    ) -> Result<Response<NodeData>, Status> {
        info!("Request get node by Id from {:?}", request.remote_addr());

        let node_id = request.into_inner().node_id;

        let node_data: crate::protobuf::acolors_proto::NodeData =
            match self.manager.get_node_by_id(node_id).await {
                Ok(c) => c.into(),
                Err(e) => {
                    return Err(Status::new(
                        Code::Unavailable,
                        format!("Node unavailable: \"{}\"", e),
                    ))
                }
            };

        let reply = node_data;

        Ok(Response::new(reply))
    }
}
