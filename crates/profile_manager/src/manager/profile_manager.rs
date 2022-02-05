use std::sync::mpsc::{self};

use anyhow::{anyhow, Result};
use tokio::sync::oneshot;

use crate::{GroupData, NodeData};

use super::{producer::create_producer, reply::ProfileReply, request::ProfileRequest};

const BUFFER_SIZE: usize = 512;
#[derive(Debug)]
pub struct ProfileManager {
    sender: mpsc::SyncSender<Request>,
}

impl ProfileManager {
    pub async fn new(path: String) -> Result<ProfileManager> {
        let (sender, rx) = mpsc::sync_channel(BUFFER_SIZE);

        create_producer(rx, path).await;

        Ok(ProfileManager { sender })
    }

    fn send_request(&self, content: ProfileRequest) -> Result<oneshot::Receiver<ProfileReply>> {
        let (sender, receiver) = oneshot::channel();
        let request = Request { sender, content };
        self.sender.send(request)?;
        Ok(receiver)
    }

    pub async fn count_groups(&self) -> Result<usize> {
        let content = ProfileRequest::CountGroups;
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::CountGroups(c) => Ok(c),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }

    pub async fn list_all_groups(&self) -> Result<Vec<GroupData>> {
        let content = ProfileRequest::ListAllGroups;
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::ListAllGroups(group_list) => Ok(group_list),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }

    pub async fn count_nodes(&self, group_id: i32) -> Result<usize> {
        let content = ProfileRequest::CountNodes(group_id);
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::CountNodes(c) => Ok(c),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }

    pub async fn list_all_nodes(&self, group_id: i32) -> Result<Vec<NodeData>> {
        let content = ProfileRequest::ListAllNodes(group_id);
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::ListAllNodes(node_list) => Ok(node_list),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }
    pub async fn get_group_by_id(&self, group_id: i32) -> Result<GroupData> {
        let content = ProfileRequest::GetGroupById(group_id);
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::GetGroupById(group_data) => Ok(group_data),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }
    pub async fn get_node_by_id(&self, node_id: i32) -> Result<NodeData> {
        let content = ProfileRequest::GetNodeById(node_id);
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::GetNodeById(node_data) => Ok(node_data),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }
    pub async fn set_group_by_id(&self, group_id: i32, group_data: GroupData) -> Result<()> {
        let content = ProfileRequest::SetGroupById(group_id, group_data);
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::SetGroupById => Ok(()),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }
    pub async fn set_node_by_id(&self, node_id: i32, node_data: NodeData) -> Result<()> {
        let content = ProfileRequest::SetNodeById(node_id, node_data);
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::SetNodeById => Ok(()),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }
    pub async fn append_group(&self, group_data: GroupData) -> Result<()> {
        let content = ProfileRequest::AppendGroup(group_data);
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::AppendGroup => Ok(()),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }
    pub async fn append_node(&self, node_id: i32, node_data: NodeData) -> Result<()> {
        let content = ProfileRequest::AppendNode(node_id, node_data);
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::AppendNode => Ok(()),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }
    pub async fn remove_group_by_id(&self, group_id: i32) -> Result<()> {
        let content = ProfileRequest::RemoveGroupById(group_id);
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::RemoveGroupById => Ok(()),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }
    pub async fn remove_node_by_id(&self, node_id: i32) -> Result<()> {
        let content = ProfileRequest::RemoveNodeById(node_id);
        let receiver = self.send_request(content)?;

        match receiver.await? {
            ProfileReply::RemoveNodeById => Ok(()),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub sender: oneshot::Sender<ProfileReply>,
    pub content: ProfileRequest,
}
