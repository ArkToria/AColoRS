use std::sync::mpsc::{self, Receiver};

use anyhow::{anyhow, Result};
use rusqlite::Connection;
use spdlog::{debug, error, info};
use tokio::{
    sync::oneshot::{self},
    task,
};

use crate::{table_member::traits::AColoRSListModel, GroupData, Profile};

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
        let profile = Profile::new(connection);

        while let Ok(request) = receiver.recv() {
            try_reply(request, &profile);
        }
        Ok(())
    });
}

fn try_reply(request: Request, profile: &Profile) {
    debug!("Got = {:?}", request);
    let (sender, request) = (request.sender, request.content);
    debug!("{:?}/{:?}", sender, request);

    match request {
        ProfileRequest::CountGroups => count_group_reply(profile, sender),
        ProfileRequest::ListAllGroups => list_all_group_reply(profile, sender),
    }
}

fn list_all_group_reply(profile: &Profile, sender: oneshot::Sender<ProfileReply>) {
    let groups = profile.group_list.list_all_nodes();

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
