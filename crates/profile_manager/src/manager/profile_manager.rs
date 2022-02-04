use std::sync::mpsc::{self, Receiver};

use anyhow::{anyhow, Result};
use rusqlite::Connection;
use spdlog::{error, info};
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

        tokio::spawn(producer(rx, path));

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
            ProfileReply::CountGroups(c) => return Ok(c),
            _ => {
                unreachable!()
            }
        };
    }
}

#[derive(Debug)]
struct Request {
    pub sender: oneshot::Sender<ProfileReply>,
    pub content: ProfileRequest,
}
async fn producer(rx: Receiver<Request>, path: String) -> Result<()> {
    let receiver = rx;
    task::spawn_blocking(move || -> Result<()> {
        let connection = create_connection(path)?;
        let profile = Profile::new(connection);
        while let Ok(request) = receiver.recv() {
            info!("Got = {:?}", request);
            let sender = request.sender;
            let request = request.content;
            match request {
                ProfileRequest::CountGroups => {
                    let count = count_groups(&profile)?;
                    if let Err(p) = sender.send(ProfileReply::CountGroups(count)) {
                        info!("Reply failed: \"{:?}\"", p);
                    }
                }
                ProfileRequest::Exit => {
                    break;
                }
            }
        }
        Ok(())
    })
    .await?
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
