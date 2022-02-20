use std::sync::Arc;

use spdlog::info;

use tonic::{Request, Response, Status};

use core_protobuf::acolors_proto::{profile_manager_server::ProfileManager, *};
use profile_manager::{self, ProfileTaskProducer};
use serialize_tool::serialize::serializetool::{decode_outbound_from_url, get_nodes_from_base64};
use utils::net::get_http_content;

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

        let count = self
            .manager
            .count_groups()
            .await
            .map_err(|e| Status::not_found(format!("Count unavailable: \"{}\"", e)))?;

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

        let group_list: Vec<core_protobuf::acolors_proto::GroupData> = self
            .manager
            .list_all_groups()
            .await
            .map(|c| c.into_iter().map(|group| group.into()).collect())
            .map_err(|e| Status::not_found(format!("Groups unavailable: \"{}\"", e)))?;

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

        let count = self
            .manager
            .count_nodes(group_id)
            .await
            .map_err(|e| Status::not_found(format!("Groups unavailable: \"{}\"", e)))?;

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

        let group_list: Vec<core_protobuf::acolors_proto::NodeData> = self
            .manager
            .list_all_nodes(group_id)
            .await
            .map(|c| c.into_iter().map(|group| group.into()).collect())
            .map_err(|e| Status::not_found(format!("Nodes unavailable: \"{}\"", e)))?;

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

        let group_data: core_protobuf::acolors_proto::GroupData = self
            .manager
            .get_group_by_id(group_id)
            .await
            .map(|c| c.into())
            .map_err(|e| Status::not_found(format!("Group unavailable: \"{}\"", e)))?;

        let reply = group_data;

        Ok(Response::new(reply))
    }
    async fn get_node_by_id(
        &self,
        request: Request<GetNodeByIdRequest>,
    ) -> Result<Response<NodeData>, Status> {
        info!("Request get node by Id from {:?}", request.remote_addr());

        let node_id = request.into_inner().node_id;

        let node_data: core_protobuf::acolors_proto::NodeData = self
            .manager
            .get_node_by_id(node_id)
            .await
            .map(|c| c.into())
            .map_err(|e| Status::not_found(format!("Node unavailable: \"{}\"", e)))?;

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
        let data: GroupData = inner
            .data
            .ok_or_else(|| Status::invalid_argument("No group data"))?;

        self.manager
            .set_group_by_id(group_id, data.into())
            .await
            .map_err(|e| Status::aborted(format!("Group unavailable: \"{}\"", e)))?;

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
        let data: NodeData = inner
            .data
            .ok_or_else(|| Status::invalid_argument("No node data"))?;

        self.manager
            .set_node_by_id(node_id, data.into())
            .await
            .map_err(|e| Status::aborted(format!("Node unavailable: \"{}\"", e)))?;

        let reply = SetNodeByIdReply {};

        Ok(Response::new(reply))
    }

    async fn set_node_by_url(
        &self,
        request: Request<SetNodeByUrlRequest>,
    ) -> Result<Response<SetNodeByUrlReply>, Status> {
        info!("Request set node by url from {:?}", request.remote_addr());

        let inner = request.into_inner();
        let node_id = inner.node_id;
        let url = inner.url;

        let node_data = decode_outbound_from_url(url)
            .map_err(|e| Status::invalid_argument(format!("Decode error: \"{}\"", e)))?;

        self.manager
            .set_node_by_id(node_id, node_data)
            .await
            .map_err(|e| Status::aborted(format!("Node unavailable: \"{}\"", e)))?;

        let reply = SetNodeByUrlReply {};

        Ok(Response::new(reply))
    }

    async fn append_group(
        &self,
        request: Request<AppendGroupRequest>,
    ) -> Result<Response<AppendGroupReply>, Status> {
        info!("Request append group from {:?}", request.remote_addr());

        let inner = request.into_inner();
        let data: GroupData = inner
            .data
            .ok_or_else(|| Status::invalid_argument("No group data"))?;

        self.manager
            .append_group(data.into())
            .await
            .map_err(|e| Status::aborted(format!("Group unavailable: \"{}\"", e)))?;

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
        let data: NodeData = inner
            .data
            .ok_or_else(|| Status::invalid_argument("No node data"))?;

        self.manager
            .append_node(group_id, data.into())
            .await
            .map_err(|e| Status::aborted(format!("Node unavailable: \"{}\"", e)))?;

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

        let node_data = decode_outbound_from_url(url)
            .map_err(|e| Status::invalid_argument(format!("Decode error: \"{}\"", e)))?;

        self.manager
            .append_node(group_id, node_data)
            .await
            .map_err(|e| Status::aborted(format!("Node unavailable: \"{}\"", e)))?;

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

        self.manager
            .remove_group_by_id(group_id)
            .await
            .map_err(|e| Status::aborted(format!("Group unavailable: \"{}\"", e)))?;

        let reply = RemoveGroupByIdReply {};

        Ok(Response::new(reply))
    }
    async fn remove_node_by_id(
        &self,
        request: Request<RemoveNodeByIdRequest>,
    ) -> Result<Response<RemoveNodeByIdReply>, Status> {
        info!("Request remove node by Id from {:?}", request.remote_addr());

        let node_id = request.into_inner().node_id;

        self.manager
            .remove_node_by_id(node_id)
            .await
            .map_err(|e| Status::aborted(format!("Node unavailable: \"{}\"", e)))?;

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

        let group_data = self
            .manager
            .get_group_by_id(group_id)
            .await
            .map_err(|e| Status::aborted(format!("Group unavailable: \"{}\"", e)))?;

        let base64: String = get_http_content(group_data.url)
            .await
            .map(|s| s.lines().map(|line| line.trim()).collect())
            .map_err(|e| Status::invalid_argument(format!("Url unavailable: \"{}\"", e)))?;

        let nodes = get_nodes_from_base64(&base64)
            .map_err(|e| Status::aborted(format!("Nodes url parsing error\"{}\"", e)))?;

        self.manager
            .update_group_by_id(group_id, nodes)
            .await
            .map_err(|e| Status::unavailable(format!("Group unavailable: \"{}\"", e)))?;

        let reply = UpdateGroupByIdReply {};

        Ok(Response::new(reply))
    }
}
