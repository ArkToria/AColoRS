use std::sync::mpsc::{self, Receiver};

use anyhow::{anyhow, Result};
use rusqlite::Connection;
use spdlog::{debug, error, info};
use tokio::{
    sync::oneshot::{self},
    task,
};

use crate::{table_member::traits::AColoRSListModel, GroupData, NodeData, Profile};

use super::{reply::ProfileReply, request::ProfileRequest};

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
    fn send_request(&self, content: ProfileRequest) -> Result<oneshot::Receiver<ProfileReply>> {
        let (sender, receiver) = oneshot::channel();
        let request = Request { sender, content };
        self.sender.send(request)?;
        Ok(receiver)
    }
}

#[derive(Debug)]
struct Request {
    pub sender: oneshot::Sender<ProfileReply>,
    pub content: ProfileRequest,
}

async fn create_producer(rx: Receiver<Request>, path: String) {
    task::spawn_blocking(move || -> Result<()> {
        let receiver = rx;
        let connection = create_connection(path)?;
        let mut profile = Profile::new(connection);

        while let Ok(request) = receiver.recv() {
            try_reply(request, &mut profile);
        }
        Ok(())
    });
}

fn try_reply(request: Request, profile: &mut Profile) {
    debug!("Got = {:?}", request);
    let (sender, request) = (request.sender, request.content);
    debug!("{:?}/{:?}", sender, request);

    match request {
        ProfileRequest::CountGroups => count_group_reply(profile, sender),
        ProfileRequest::ListAllGroups => list_all_group_reply(profile, sender),
        ProfileRequest::CountNodes(group_id) => count_node_reply(profile, sender, group_id),
        ProfileRequest::ListAllNodes(group_id) => list_all_node_reply(profile, sender, group_id),
        ProfileRequest::GetGroupById(group_id) => get_group_by_id_reply(profile, sender, group_id),
        ProfileRequest::GetNodeById(node_id) => get_node_by_id_reply(profile, sender, node_id),
        ProfileRequest::SetGroupById(group_id, group_data) => {
            set_group_by_id_reply(profile, sender, group_id, group_data)
        }
        ProfileRequest::SetNodeById(node_id, node_data) => {
            set_node_by_id_reply(profile, sender, node_id, node_data)
        }
    }
}

fn set_group_by_id_reply(
    profile: &mut Profile,
    sender: oneshot::Sender<ProfileReply>,
    group_id: i32,
    group_data: GroupData,
) {
    let group = profile.group_list.set(group_id as usize, &group_data);

    match group {
        Ok(_) => {
            debug!("Set group By ID : {}", group_id);
            try_send(sender, ProfileReply::SetGroupById);
        }
        Err(e) => {
            debug!("Set group By ID Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
        }
    }
}
fn set_node_by_id_reply(
    profile: &mut Profile,
    sender: oneshot::Sender<ProfileReply>,
    node_id: i32,
    node_data: NodeData,
) {
    let mut group = profile.group_list.default_group();

    let node = group.set(node_id as usize, &node_data);

    match node {
        Ok(_) => {
            debug!("Set node By ID : {}", node_id);
            try_send(sender, ProfileReply::SetNodeById);
        }
        Err(e) => {
            debug!("Set node By ID Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
        }
    }
}

fn get_group_by_id_reply(profile: &Profile, sender: oneshot::Sender<ProfileReply>, group_id: i32) {
    let group = profile.group_list.query(group_id as usize);

    match group {
        Ok(g) => {
            debug!("Query group By ID : {}", group_id);
            let group = g.to_data();
            try_send(sender, ProfileReply::GetGroupById(group));
        }
        Err(e) => {
            debug!("Query group By ID Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
        }
    }
}
fn get_node_by_id_reply(profile: &Profile, sender: oneshot::Sender<ProfileReply>, node_id: i32) {
    let group = profile.group_list.default_group();

    let node = group.query(node_id as usize);

    match node {
        Ok(g) => {
            debug!("Query node By ID : {}", node_id);
            let node = g.to_data();
            try_send(sender, ProfileReply::GetNodeById(node));
        }
        Err(e) => {
            debug!("Query node By ID Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
        }
    }
}

fn list_all_group_reply(profile: &Profile, sender: oneshot::Sender<ProfileReply>) {
    let groups = profile.group_list.list_all_groups();

    match groups {
        Ok(g) => {
            debug!("List all groups");
            let group_list = g.into_iter().map(|group| group.to_data()).collect();
            try_send(sender, ProfileReply::ListAllGroups(group_list));
        }
        Err(e) => {
            debug!("List all groups Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
        }
    }
}

fn count_group_reply(profile: &Profile, sender: oneshot::Sender<ProfileReply>) {
    let count = profile.group_list.size();
    match count {
        Ok(c) => {
            debug!("Group count:{}", c);
            try_send(sender, ProfileReply::CountGroups(c));
        }
        Err(e) => {
            debug!("Group count Error : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
        }
    }
}

fn list_all_node_reply(profile: &Profile, sender: oneshot::Sender<ProfileReply>, group_id: i32) {
    let group = match profile.group_list.query(group_id as usize) {
        Ok(group) => group,
        Err(e) => {
            debug!("List all nodes Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
            return;
        }
    };

    let nodes = group.list_all_nodes();

    match nodes {
        Ok(g) => {
            debug!("List all nodes");
            let node_list = g.into_iter().map(|node| node.to_data()).collect();
            try_send(sender, ProfileReply::ListAllNodes(node_list));
        }
        Err(e) => {
            debug!("List all nodes Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
        }
    }
}

fn count_node_reply(profile: &Profile, sender: oneshot::Sender<ProfileReply>, group_id: i32) {
    let group = match profile.group_list.query(group_id as usize) {
        Ok(group) => group,
        Err(e) => {
            debug!("List all nodes Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
            return;
        }
    };

    let count = group.size();
    match count {
        Ok(c) => {
            debug!("Node count:{}", c);
            try_send(sender, ProfileReply::CountNodes(c));
        }
        Err(e) => {
            debug!("Node count Error : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
        }
    }
}

fn try_send(sender: oneshot::Sender<ProfileReply>, reply: ProfileReply) {
    if let Err(p) = sender.send(reply) {
        info!("Reply failed: \"{:?}\"", p);
    }
}
fn create_connection(path: String) -> Result<Connection> {
    match Connection::open(path) {
        Ok(c) => Ok(c),
        Err(e) => {
            error!("Channel open failed: {}", e);
            return Err(anyhow!("{}", e));
        }
    }
}
