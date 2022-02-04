use std::sync::mpsc::{self, Receiver};

use anyhow::{anyhow, Result};
use rusqlite::Connection;
use spdlog::{debug, error, info};
use tokio::{
    sync::oneshot::{self},
    task,
};

use crate::{table_member::traits::AColoRSListModel, Profile};

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
        let (sender, receiver) = oneshot::channel();

        let request = Request {
            sender,
            content: ProfileRequest::CountGroups,
        };
        self.sender.send(request)?;

        match receiver.await? {
            ProfileReply::CountGroups(c) => Ok(c),
            ProfileReply::Error(e) => Err(anyhow!("{}", e)),

            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct Request {
    pub sender: oneshot::Sender<ProfileReply>,
    pub content: ProfileRequest,
}

async fn create_producer(rx: Receiver<Request>, path: String) {
    let receiver = rx;
    task::spawn_blocking(move || -> Result<()> {
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
    }
}

fn count_group_reply(profile: &Profile, sender: oneshot::Sender<ProfileReply>) {
    let count = count_groups(profile);
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
fn count_groups(profile: &Profile) -> Result<usize> {
    profile.group_list.size()
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
