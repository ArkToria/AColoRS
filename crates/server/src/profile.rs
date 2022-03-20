use std::sync::Arc;

use acolors_signal::send_or_warn_print;
use spdlog::{debug, error};

use tokio::sync::{broadcast, RwLock};
use tonic::{Request, Response, Status};

use core_protobuf::acolors_proto::{profile_manager_server::ProfileManager, *};
use profile_manager::{self, Group, Profile};
use serialize_tool::serialize::serializetool::{decode_outbound_from_url, get_nodes_from_base64};
use utils::net::get_http_content;

type InboundsLock = Arc<RwLock<config_manager::Inbounds>>;

#[derive(Debug)]
pub struct AColoRSProfile {
    profile: Arc<Profile>,
    inbounds: InboundsLock,
    signal_sender: broadcast::Sender<profile_manager::AColorSignal>,
}
impl AColoRSProfile {
    pub fn new(
        profile: Arc<Profile>,
        inbounds: InboundsLock,
        signal_sender: broadcast::Sender<profile_manager::AColorSignal>,
    ) -> Self {
        Self {
            profile,
            inbounds,
            signal_sender,
        }
    }
    async fn count_groups(&self) -> Result<i64, Status> {
        let count = self
            .profile
            .group_list
            .size()
            .await
            .map_err(|e| Status::not_found(format!("Count unavailable: \"{}\"", e)))?;
        Ok(count)
    }

    async fn list_all_groups(&self) -> Result<Vec<GroupData>, Status> {
        let group_list = self
            .profile
            .group_list
            .list_all_groups()
            .await
            .map(|c| c.into_iter().map(|group| group.to_data().into()).collect())
            .map_err(|e| Status::not_found(format!("Groups unavailable: \"{}\"", e)))?;
        Ok(group_list)
    }
    async fn count_nodes(&self, group_id: i64) -> Result<i64, Status> {
        let count = self
            .profile
            .group_list
            .query(group_id as i64)
            .await
            .map_err(|e| Status::not_found(format!("Groups unavailable: \"{}\"", e)))?
            .size()
            .await
            .map_err(|e| Status::aborted(format!("Count Nodes Failed: \"{}\"", e)))?;
        Ok(count)
    }
    async fn list_all_nodes(&self, group_id: i64) -> Result<Vec<NodeData>, Status> {
        let node_list = self
            .profile
            .group_list
            .query(group_id as i64)
            .await
            .map_err(|e| Status::not_found(format!("Groups unavailable: \"{}\"", e)))?
            .list_all_nodes()
            .await
            .map(|c| c.into_iter().map(|group| group.to_data().into()).collect())
            .map_err(|e| Status::not_found(format!("Nodes unavailable: \"{}\"", e)))?;
        Ok(node_list)
    }
    async fn get_group_by_id(&self, group_id: i64) -> Result<profile_manager::Group, Status> {
        let group = self
            .profile
            .group_list
            .query(group_id as i64)
            .await
            .map_err(|e| Status::not_found(format!("Group unavailable: \"{}\"", e)))?;
        Ok(group)
    }
    async fn get_node_by_id(&self, node_id: i64) -> Result<profile_manager::Node, Status> {
        let node = self
            .profile
            .group_list
            .default_group()
            .query(node_id as i64)
            .await
            .map_err(|e| Status::not_found(format!("Node unavailable: \"{}\"", e)))?;
        Ok(node)
    }
    async fn set_group_by_id(&self, group_id: i64, data: GroupData) -> Result<(), Status> {
        self.profile
            .group_list
            .set(group_id as i64, data.into())
            .await
            .map_err(|e| Status::aborted(format!("Group unavailable: \"{}\"", e)))?;
        send_or_warn_print(
            &self.signal_sender,
            acolors_signal::AColorSignal::SetGroupById(group_id),
        );
        Ok(())
    }
    async fn set_node_by_id(&self, node_id: i64, data: NodeData) -> Result<(), Status> {
        self.profile
            .group_list
            .default_group()
            .set(node_id as i64, data.into())
            .await
            .map_err(|e| Status::aborted(format!("Node unavailable: \"{}\"", e)))?;
        send_or_warn_print(
            &self.signal_sender,
            acolors_signal::AColorSignal::SetNodeById(node_id),
        );
        Ok(())
    }
    async fn append_group(&self, data: GroupData) -> Result<i64, Status> {
        let group_id = self
            .profile
            .group_list
            .append(data.into())
            .await
            .map_err(|e| Status::aborted(format!("Group unavailable: \"{}\"", e)))?;
        send_or_warn_print(
            &self.signal_sender,
            acolors_signal::AColorSignal::AppendGroup(group_id),
        );
        Ok(group_id)
    }
    async fn append_node(&self, group_id: i64, data: NodeData) -> Result<i64, Status> {
        let node_id = self
            .profile
            .group_list
            .query(group_id as i64)
            .await
            .map_err(|e| Status::not_found(format!("Groups unavailable: \"{}\"", e)))?
            .append(data.into())
            .await
            .map_err(|e| Status::aborted(format!("Node unavailable: \"{}\"", e)))?;
        send_or_warn_print(
            &self.signal_sender,
            acolors_signal::AColorSignal::AppendNode(group_id, node_id),
        );
        Ok(node_id)
    }
    async fn remove_group_by_id(&self, group_id: i64) -> Result<(), Status> {
        self.profile
            .group_list
            .remove(group_id as i64)
            .await
            .map_err(|e| Status::aborted(format!("Group unavailable: \"{}\"", e)))?;

        send_or_warn_print(
            &self.signal_sender,
            acolors_signal::AColorSignal::RemoveGroupById(group_id),
        );
        Ok(())
    }
    async fn remove_node_by_id(&self, node_id: i64) -> Result<(), Status> {
        self.profile
            .group_list
            .default_group()
            .remove(node_id as i64)
            .await
            .map_err(|e| Status::aborted(format!("Node unavailable: \"{}\"", e)))?;
        send_or_warn_print(
            &self.signal_sender,
            acolors_signal::AColorSignal::RemoveNodeById(node_id),
        );
        Ok(())
    }
    async fn empty_group(&self, group: &Group) -> Result<(), Status> {
        group
            .remove_all_nodes()
            .await
            .map_err(|e| Status::aborted(&format!("Empty Group Error: {}", e)))?;
        send_or_warn_print(
            &self.signal_sender,
            acolors_signal::AColorSignal::EmptyGroup(group.data().id),
        );
        Ok(())
    }
    async fn update_group_by_id(&self, group_id: i64, use_proxy: bool) -> Result<(), Status> {
        let proxy = if use_proxy {
            let inbounds = &*self.inbounds.read().await;
            let http_inbound = &inbounds.http;
            http_inbound
                .as_ref()
                .map(|inbound| format!("http://{}:{}", inbound.listen, inbound.port))
                .unwrap_or_default()
        } else {
            String::new()
        };

        let group = self.get_group_by_id(group_id).await?;

        let base64: String = get_http_content(group.data().url.clone(), &proxy)
            .await
            .map(|s| s.lines().map(|line| line.trim()).collect())
            .map_err(|e| Status::invalid_argument(format!("Url unavailable: \"{}\"", e)))?;

        let nodes = get_nodes_from_base64(&base64)
            .map_err(|e| Status::aborted(format!("Nodes url parsing error\"{}\"", e)))?;

        self.empty_group(&group).await?;
        for node in nodes {
            group.append(node).await.unwrap_or_else(|e| {
                error!("Insert Node Failed: {}", e);
                0
            });
        }
        send_or_warn_print(
            &self.signal_sender,
            acolors_signal::AColorSignal::UpdateGroup(group.data().id),
        );
        Ok(())
    }
}

#[tonic::async_trait]
impl ProfileManager for AColoRSProfile {
    async fn count_groups(
        &self,
        request: Request<CountGroupsRequest>,
    ) -> Result<Response<CountGroupsReply>, Status> {
        debug!("Request count groups from {:?}", request.remote_addr());

        let count = self.count_groups().await?;

        let reply = CountGroupsReply {
            count: count as u64,
        };
        Ok(Response::new(reply))
    }

    async fn list_all_groups(
        &self,
        request: Request<ListAllGroupsRequest>,
    ) -> Result<Response<GroupList>, Status> {
        debug!("Request list all groups from {:?}", request.remote_addr());

        let group_list = self.list_all_groups().await?;

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
        debug!("Request count nodes from {:?}", request.remote_addr());

        let group_id = request.into_inner().group_id;

        let count = self.count_nodes(group_id).await?;

        let reply = CountNodesReply {
            count: count as u64,
        };
        Ok(Response::new(reply))
    }

    async fn list_all_nodes(
        &self,
        request: Request<ListAllNodesRequest>,
    ) -> Result<Response<NodeList>, Status> {
        debug!("Request list all nodes from {:?}", request.remote_addr());

        let group_id = request.into_inner().group_id;

        let node_list = self.list_all_nodes(group_id).await?;

        let length = node_list.len();
        let reply = NodeList {
            length: length as u64,
            entries: node_list,
        };
        Ok(Response::new(reply))
    }

    async fn get_group_by_id(
        &self,
        request: Request<GetGroupByIdRequest>,
    ) -> Result<Response<GroupData>, Status> {
        debug!("Request get group by Id from {:?}", request.remote_addr());

        let group_id = request.into_inner().group_id;

        let group = self.get_group_by_id(group_id).await?;

        let reply = group.to_data().into();

        Ok(Response::new(reply))
    }
    async fn get_node_by_id(
        &self,
        request: Request<GetNodeByIdRequest>,
    ) -> Result<Response<NodeData>, Status> {
        debug!("Request get node by Id from {:?}", request.remote_addr());

        let node_id = request.into_inner().node_id;

        let node = self.get_node_by_id(node_id).await?;

        let reply = node.to_data().into();

        Ok(Response::new(reply))
    }

    async fn set_group_by_id(
        &self,
        request: Request<SetGroupByIdRequest>,
    ) -> Result<Response<SetGroupByIdReply>, Status> {
        debug!("Request set group by Id from {:?}", request.remote_addr());

        let inner = request.into_inner();
        let group_id = inner.group_id;
        let data: GroupData = inner
            .data
            .ok_or_else(|| Status::invalid_argument("No group data"))?;

        self.set_group_by_id(group_id, data).await?;

        let reply = SetGroupByIdReply {};

        Ok(Response::new(reply))
    }
    async fn set_node_by_id(
        &self,
        request: Request<SetNodeByIdRequest>,
    ) -> Result<Response<SetNodeByIdReply>, Status> {
        debug!("Request set node by Id from {:?}", request.remote_addr());

        let inner = request.into_inner();
        let node_id = inner.node_id;
        let data: NodeData = inner
            .data
            .ok_or_else(|| Status::invalid_argument("No node data"))?;

        self.set_node_by_id(node_id, data).await?;

        let reply = SetNodeByIdReply {};

        Ok(Response::new(reply))
    }

    async fn set_node_by_url(
        &self,
        request: Request<SetNodeByUrlRequest>,
    ) -> Result<Response<SetNodeByUrlReply>, Status> {
        debug!("Request set node by url from {:?}", request.remote_addr());

        let inner = request.into_inner();
        let node_id = inner.node_id;
        let url = inner.url;

        let node_data = decode_outbound_from_url(url)
            .map_err(|e| Status::invalid_argument(format!("Decode error: \"{}\"", e)))?;

        self.set_node_by_id(node_id, node_data.into()).await?;

        let reply = SetNodeByUrlReply {};

        Ok(Response::new(reply))
    }

    async fn append_group(
        &self,
        request: Request<AppendGroupRequest>,
    ) -> Result<Response<AppendGroupReply>, Status> {
        debug!("Request append group from {:?}", request.remote_addr());

        let inner = request.into_inner();
        let data: GroupData = inner
            .data
            .ok_or_else(|| Status::invalid_argument("No group data"))?;

        let group_id = self.append_group(data).await?;

        let reply = AppendGroupReply { group_id };

        Ok(Response::new(reply))
    }

    async fn append_node(
        &self,
        request: Request<AppendNodeRequest>,
    ) -> Result<Response<AppendNodeReply>, Status> {
        debug!(
            "Request append node by group id from {:?}",
            request.remote_addr()
        );

        let inner = request.into_inner();
        let group_id = inner.group_id;
        let data: NodeData = inner
            .data
            .ok_or_else(|| Status::invalid_argument("No node data"))?;

        let node_id = self.append_node(group_id, data).await?;

        let reply = AppendNodeReply { node_id };

        Ok(Response::new(reply))
    }

    async fn append_node_by_url(
        &self,
        request: Request<AppendNodeByUrlRequest>,
    ) -> Result<Response<AppendNodeByUrlReply>, Status> {
        debug!(
            "Request append node by group id from {:?}",
            request.remote_addr()
        );

        let inner = request.into_inner();
        let group_id = inner.group_id;
        let url = inner.url;

        let node_data = decode_outbound_from_url(url)
            .map_err(|e| Status::invalid_argument(format!("Decode error: \"{}\"", e)))?;

        let node_id = self.append_node(group_id, node_data.into()).await?;

        let reply = AppendNodeByUrlReply { node_id };

        Ok(Response::new(reply))
    }

    async fn remove_group_by_id(
        &self,
        request: Request<RemoveGroupByIdRequest>,
    ) -> Result<Response<RemoveGroupByIdReply>, Status> {
        debug!(
            "Request remove group by Id from {:?}",
            request.remote_addr()
        );

        let group_id = request.into_inner().group_id;

        self.remove_group_by_id(group_id).await?;

        let reply = RemoveGroupByIdReply {};

        Ok(Response::new(reply))
    }
    async fn remove_node_by_id(
        &self,
        request: Request<RemoveNodeByIdRequest>,
    ) -> Result<Response<RemoveNodeByIdReply>, Status> {
        debug!("Request remove node by Id from {:?}", request.remote_addr());

        let node_id = request.into_inner().node_id;

        self.remove_node_by_id(node_id).await?;

        let reply = RemoveNodeByIdReply {};

        Ok(Response::new(reply))
    }

    async fn update_group_by_id(
        &self,
        request: Request<UpdateGroupByIdRequest>,
    ) -> Result<Response<UpdateGroupByIdReply>, Status> {
        debug!(
            "Request update group by Id from {:?}",
            request.remote_addr()
        );

        let inner = request.into_inner();
        let group_id = inner.group_id;
        let use_proxy = inner.use_proxy;

        self.update_group_by_id(group_id, use_proxy).await?;

        let reply = UpdateGroupByIdReply {};

        Ok(Response::new(reply))
    }
    async fn empty_group_by_id(
        &self,
        request: Request<EmptyGroupByIdRequest>,
    ) -> Result<Response<EmptyGroupByIdReply>, Status> {
        debug!("Request empty group by Id from {:?}", request.remote_addr());

        let inner = request.into_inner();
        let group_id = inner.group_id;

        let group = self.get_group_by_id(group_id).await?;

        self.empty_group(&group).await?;

        let reply = EmptyGroupByIdReply {};

        Ok(Response::new(reply))
    }
}
