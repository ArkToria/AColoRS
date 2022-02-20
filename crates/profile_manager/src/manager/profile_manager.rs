use std::{
    path::Path,
    sync::mpsc::{self},
};

use anyhow::{anyhow, Result};
use tokio::sync::{broadcast, oneshot};

use core_data::{GroupData, NodeData};

use super::{consumer::create_consumer, reply::ProfileReply, request::ProfileRequest};
use acolors_signal::AColorSignal;

macro_rules! send_request {
    ( $self:expr, $request:expr, ProfileReply::$reply:ident($($field:ident),*) ) => {{
        let receiver = $self.send_request($request)?;

        match receiver.await? {
            ProfileReply::$reply($($field),*) => Ok($($field),*),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }};
    ( $self:expr, $request:expr, ProfileReply::$reply:ident ) => {{
        let receiver = $self.send_request($request)?;

        match receiver.await? {
            ProfileReply::$reply => Ok(()),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }};
}

#[derive(Debug)]
pub struct ProfileTaskProducer {
    sender: mpsc::SyncSender<Request>,
}

impl ProfileTaskProducer {
    pub async fn new<P: AsRef<Path>>(
        path: P,
        signal_sender: broadcast::Sender<AColorSignal>,
        buffer_size: usize,
    ) -> Result<ProfileTaskProducer> {
        let (sender, rx) = mpsc::sync_channel(buffer_size);

        create_consumer(rx, signal_sender.clone(), path).await;

        Ok(ProfileTaskProducer { sender })
    }

    fn send_request(&self, content: ProfileRequest) -> Result<oneshot::Receiver<ProfileReply>> {
        let (sender, receiver) = oneshot::channel();
        let request = Request { sender, content };
        self.sender.send(request)?;
        Ok(receiver)
    }

    pub async fn count_groups(&self) -> Result<usize> {
        send_request!(
            self,
            ProfileRequest::CountGroups,
            ProfileReply::CountGroups(value)
        )
    }

    pub async fn list_all_groups(&self) -> Result<Vec<GroupData>> {
        send_request!(
            self,
            ProfileRequest::ListAllGroups,
            ProfileReply::ListAllGroups(value)
        )
    }

    pub async fn count_nodes(&self, group_id: i32) -> Result<usize> {
        send_request!(
            self,
            ProfileRequest::CountNodes(group_id),
            ProfileReply::CountNodes(value)
        )
    }

    pub async fn list_all_nodes(&self, group_id: i32) -> Result<Vec<NodeData>> {
        send_request!(
            self,
            ProfileRequest::ListAllNodes(group_id),
            ProfileReply::ListAllNodes(value)
        )
    }
    pub async fn get_group_by_id(&self, group_id: i32) -> Result<GroupData> {
        send_request!(
            self,
            ProfileRequest::GetGroupById(group_id),
            ProfileReply::GetGroupById(value)
        )
    }
    pub async fn get_node_by_id(&self, node_id: i32) -> Result<NodeData> {
        send_request!(
            self,
            ProfileRequest::GetNodeById(node_id),
            ProfileReply::GetNodeById(value)
        )
    }
    pub async fn set_group_by_id(&self, group_id: i32, group_data: GroupData) -> Result<()> {
        send_request!(
            self,
            ProfileRequest::SetGroupById(group_id, group_data),
            ProfileReply::SetGroupById
        )
    }
    pub async fn set_node_by_id(&self, node_id: i32, node_data: NodeData) -> Result<()> {
        send_request!(
            self,
            ProfileRequest::SetNodeById(node_id, node_data),
            ProfileReply::SetNodeById
        )
    }
    pub async fn append_group(&self, group_data: GroupData) -> Result<()> {
        send_request!(
            self,
            ProfileRequest::AppendGroup(group_data),
            ProfileReply::AppendGroup
        )
    }
    pub async fn append_node(&self, node_id: i32, node_data: NodeData) -> Result<()> {
        send_request!(
            self,
            ProfileRequest::AppendNode(node_id, node_data),
            ProfileReply::AppendNode
        )
    }
    pub async fn remove_group_by_id(&self, group_id: i32) -> Result<()> {
        send_request!(
            self,
            ProfileRequest::RemoveGroupById(group_id),
            ProfileReply::RemoveGroupById
        )
    }
    pub async fn remove_node_by_id(&self, node_id: i32) -> Result<()> {
        send_request!(
            self,
            ProfileRequest::RemoveNodeById(node_id),
            ProfileReply::RemoveNodeById
        )
    }

    pub async fn update_group_by_id(&self, group_id: i32, nodes: Vec<NodeData>) -> Result<()> {
        send_request!(
            self,
            ProfileRequest::UpdateGroup(group_id, nodes),
            ProfileReply::UpdateGroup
        )
    }
}

#[derive(Debug)]
pub struct Request {
    pub sender: oneshot::Sender<ProfileReply>,
    pub content: ProfileRequest,
}
