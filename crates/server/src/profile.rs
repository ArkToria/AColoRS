use std::sync::Arc;

use spdlog::info;

use tonic::{Code, Request, Response, Status};

use crate::utils::get_http_content;
use core_protobuf::acolors_proto::{profile_manager_server::ProfileManager, *};
use profile_manager::{
    self,
    serialize::serializetool::{decode_outbound_from_url, get_nodes_from_base64},
    ProfileTaskProducer,
};

#[derive(Debug)]
pub struct AColoRSProfile {
    manager: Arc<ProfileTaskProducer>,
}
impl AColoRSProfile {
    pub async fn new(manager: Arc<ProfileTaskProducer>) -> Self {
        Self { manager }
    }
}

#[tonic::async_trait]
impl ProfileManager for AColoRSProfile {
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

        let group_list: Vec<core_protobuf::acolors_proto::GroupData> =
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

        let group_list: Vec<core_protobuf::acolors_proto::NodeData> =
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

        let group_data: core_protobuf::acolors_proto::GroupData =
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

        let node_data: core_protobuf::acolors_proto::NodeData =
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

    async fn set_group_by_id(
        &self,
        request: Request<SetGroupByIdRequest>,
    ) -> Result<Response<SetGroupByIdReply>, Status> {
        info!("Request set group by Id from {:?}", request.remote_addr());

        let inner = request.into_inner();
        let group_id = inner.group_id;
        let data: GroupData = match inner.data {
            Some(d) => d,
            None => return Err(Status::invalid_argument("No group data")),
        };

        if let Err(e) = self.manager.set_group_by_id(group_id, data.into()).await {
            return Err(Status::new(
                Code::Unavailable,
                format!("Group unavailable: \"{}\"", e),
            ));
        }

        let reply = SetGroupByIdReply {};

        Ok(Response::new(reply))
    }
    async fn set_node_by_id(
        &self,
        request: Request<SetNodeByIdRequest>,
    ) -> Result<Response<SetNodeByIdReply>, Status> {
        info!("Request set node by Id from {:?}", request.remote_addr());

        let inner = request.into_inner();
        let node_id = inner.node_id;
        let data: NodeData = match inner.data {
            Some(d) => d,
            None => return Err(Status::invalid_argument("No node data")),
        };

        if let Err(e) = self.manager.set_node_by_id(node_id, data.into()).await {
            return Err(Status::new(
                Code::Unavailable,
                format!("Node unavailable: \"{}\"", e),
            ));
        }

        let reply = SetNodeByIdReply {};

        Ok(Response::new(reply))
    }

    async fn append_group(
        &self,
        request: Request<AppendGroupRequest>,
    ) -> Result<Response<AppendGroupReply>, Status> {
        info!("Request append group from {:?}", request.remote_addr());

        let inner = request.into_inner();
        let data: GroupData = match inner.data {
            Some(d) => d,
            None => return Err(Status::invalid_argument("No group data")),
        };

        if let Err(e) = self.manager.append_group(data.into()).await {
            return Err(Status::new(
                Code::Unavailable,
                format!("Group unavailable: \"{}\"", e),
            ));
        }

        let reply = AppendGroupReply {};

        Ok(Response::new(reply))
    }

    async fn append_node(
        &self,
        request: Request<AppendNodeRequest>,
    ) -> Result<Response<AppendNodeReply>, Status> {
        info!(
            "Request append node by group id from {:?}",
            request.remote_addr()
        );

        let inner = request.into_inner();
        let group_id = inner.group_id;
        let data: NodeData = match inner.data {
            Some(d) => d,
            None => return Err(Status::invalid_argument("No node data")),
        };

        if let Err(e) = self.manager.append_node(group_id, data.into()).await {
            return Err(Status::new(
                Code::Unavailable,
                format!("Node unavailable: \"{}\"", e),
            ));
        }

        let reply = AppendNodeReply {};

        Ok(Response::new(reply))
    }

    async fn append_node_by_url(
        &self,
        request: Request<AppendNodeByUrlRequest>,
    ) -> Result<Response<AppendNodeByUrlReply>, Status> {
        info!(
            "Request append node by group id from {:?}",
            request.remote_addr()
        );

        let inner = request.into_inner();
        let group_id = inner.group_id;
        let url = inner.url;

        let group_data = match decode_outbound_from_url(url) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::new(
                    Code::Unavailable,
                    format!("Decode error: \"{}\"", e),
                ));
            }
        };

        if let Err(e) = self.manager.append_node(group_id, group_data).await {
            return Err(Status::new(
                Code::Unavailable,
                format!("Node unavailable: \"{}\"", e),
            ));
        }

        let reply = AppendNodeByUrlReply {};

        Ok(Response::new(reply))
    }

    async fn remove_group_by_id(
        &self,
        request: Request<RemoveGroupByIdRequest>,
    ) -> Result<Response<RemoveGroupByIdReply>, Status> {
        info!(
            "Request remove group by Id from {:?}",
            request.remote_addr()
        );

        let group_id = request.into_inner().group_id;

        if let Err(e) = self.manager.remove_group_by_id(group_id).await {
            return Err(Status::new(
                Code::Unavailable,
                format!("Group unavailable: \"{}\"", e),
            ));
        };

        let reply = RemoveGroupByIdReply {};

        Ok(Response::new(reply))
    }
    async fn remove_node_by_id(
        &self,
        request: Request<RemoveNodeByIdRequest>,
    ) -> Result<Response<RemoveNodeByIdReply>, Status> {
        info!("Request remove node by Id from {:?}", request.remote_addr());

        let node_id = request.into_inner().node_id;

        if let Err(e) = self.manager.remove_node_by_id(node_id).await {
            return Err(Status::new(
                Code::Unavailable,
                format!("Node unavailable: \"{}\"", e),
            ));
        };

        let reply = RemoveNodeByIdReply {};

        Ok(Response::new(reply))
    }

    async fn update_group_by_id(
        &self,
        request: Request<UpdateGroupByIdRequest>,
    ) -> Result<Response<UpdateGroupByIdReply>, Status> {
        info!(
            "Request update group by Id from {:?}",
            request.remote_addr()
        );

        let inner = request.into_inner();
        let group_id = inner.group_id;

        let group_data = match self.manager.get_group_by_id(group_id).await {
            Ok(c) => c,
            Err(e) => {
                return Err(Status::new(
                    Code::Unavailable,
                    format!("Group unavailable: \"{}\"", e),
                ))
            }
        };

        let base64: String = match get_http_content(group_data.url).await {
            Ok(s) => s.lines().map(|line| line.trim()).collect(),
            Err(e) => {
                return Err(Status::new(
                    Code::Unavailable,
                    format!("Url unavailable: \"{}\"", e),
                ))
            }
        };

        let nodes = match get_nodes_from_base64(&base64) {
            Ok(n) => n,
            Err(e) => {
                return Err(Status::new(
                    Code::Unavailable,
                    format!("Nodes url parsing error\"{}\"", e),
                ))
            }
        };

        if let Err(e) = self.manager.update_group_by_id(group_id, nodes).await {
            return Err(Status::new(
                Code::Unavailable,
                format!("Group unavailable: \"{}\"", e),
            ));
        }

        let reply = UpdateGroupByIdReply {};

        Ok(Response::new(reply))
    }
}
