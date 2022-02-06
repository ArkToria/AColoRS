use std::sync::mpsc::Receiver;

use anyhow::{anyhow, Result};
use rusqlite::Connection;
use spdlog::{debug, error, info};
use tokio::{sync::oneshot, task};

use crate::{table_member::traits::AColoRSListModel, GroupData, NodeData, Profile};

use super::{profile_manager::Request, reply::ProfileReply, request::ProfileRequest};

pub async fn create_producer(rx: Receiver<Request>, path: String) {
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
        ProfileRequest::RemoveGroupById(group_id) => {
            remove_group_by_id_reply(profile, sender, group_id)
        }
        ProfileRequest::RemoveNodeById(node_id) => {
            remove_node_by_id_reply(profile, sender, node_id)
        }
        ProfileRequest::SetGroupById(group_id, group_data) => {
            set_group_by_id_reply(profile, sender, group_id, group_data)
        }
        ProfileRequest::SetNodeById(node_id, node_data) => {
            set_node_by_id_reply(profile, sender, node_id, node_data)
        }
        ProfileRequest::AppendGroup(group_data) => append_group_reply(profile, sender, group_data),
        ProfileRequest::AppendNode(group_id, node_data) => {
            append_node_reply(profile, sender, group_id, node_data)
        }
        ProfileRequest::UpdateGroup(group_id, nodes) => {
            update_group_by_id_reply(profile, sender, group_id, nodes)
        }
    }
}

fn update_group_by_id_reply(
    profile: &mut Profile,
    sender: oneshot::Sender<ProfileReply>,
    group_id: i32,
    nodes: Vec<NodeData>,
) {
    let mut group = match profile.group_list.query(group_id as usize) {
        Ok(g) => g,
        Err(e) => {
            debug!("Append node Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
            return;
        }
    };

    if let Err(e) = group.remove_all_nodes() {
        debug!("Clear group Failed : {}", e);
        try_send(sender, ProfileReply::Error(e.to_string()));
        return;
    }

    debug!("Updating group");

    let nodes = nodes;

    for node_data in nodes {
        let mut node_data = node_data.clone();
        node_data.update_create_at();
        node_data.update_modified_at();
        node_data.group_id = group_id;

        let result = group.append(&node_data);

        if let Err(e) = result {
            debug!("Update group Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
            return;
        }
    }
    try_send(sender, ProfileReply::UpdateGroup);
}
fn remove_group_by_id_reply(
    profile: &mut Profile,
    sender: oneshot::Sender<ProfileReply>,
    group_id: i32,
) {
    let result = profile.group_list.remove(group_id as usize);

    match result {
        Ok(_) => {
            debug!("Remove group By ID : {}", group_id);
            try_send(sender, ProfileReply::RemoveGroupById);
        }
        Err(e) => {
            debug!("Remove group By ID Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
        }
    }
}
fn remove_node_by_id_reply(
    profile: &mut Profile,
    sender: oneshot::Sender<ProfileReply>,
    node_id: i32,
) {
    let mut group = profile.group_list.default_group();

    let result = group.remove(node_id as usize);

    match result {
        Ok(_) => {
            debug!("Remove node By ID : {}", node_id);
            try_send(sender, ProfileReply::RemoveNodeById);
        }
        Err(e) => {
            debug!("Remove node By ID Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
        }
    }
}

fn append_group_reply(
    profile: &mut Profile,
    sender: oneshot::Sender<ProfileReply>,
    group_data: GroupData,
) {
    let group = profile.group_list.append(&group_data);

    match group {
        Ok(_) => {
            debug!("Append group");
            try_send(sender, ProfileReply::AppendGroup);
        }
        Err(e) => {
            debug!("Append group Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
        }
    }
}
fn append_node_reply(
    profile: &mut Profile,
    sender: oneshot::Sender<ProfileReply>,
    group_id: i32,
    node_data: NodeData,
) {
    let mut group = match profile.group_list.query(group_id as usize) {
        Ok(g) => g,
        Err(e) => {
            debug!("Append node Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
            return;
        }
    };

    let node = group.append(&node_data);

    match node {
        Ok(_) => {
            debug!("Append node");
            try_send(sender, ProfileReply::AppendNode);
        }
        Err(e) => {
            debug!("Append node Failed : {}", e);
            try_send(sender, ProfileReply::Error(e.to_string()));
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
