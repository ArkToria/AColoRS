use std::{sync::Arc, time::Duration};

use core_protobuf::{
    acolors_proto::TrafficInfo,
    v2ray_api_proto::{stats_service_client::StatsServiceClient, GetStatsRequest},
};
use tokio::sync::Mutex;

use tonic::{transport::Channel, Status};

use crate::BUFFER_SIZE;

#[derive(Debug)]
pub struct TrafficInfoUpdater {
    info: Arc<Mutex<TrafficInfo>>,
    stop_sender: Option<tokio::sync::mpsc::Sender<()>>,
}
async fn call_api(
    client: &mut StatsServiceClient<Channel>,
    tag: &str,
    r#type: &str,
) -> Result<i64, Status> {
    let request = GetStatsRequest {
        name: format!("outbound>>>{}>>>traffic>>>{}", tag, r#type),
        reset: false,
    };
    let response = client.get_stats(request).await?;
    Ok(response
        .into_inner()
        .stat
        .map(|stat| stat.value)
        .unwrap_or(0))
}
async fn updater(
    mut client: StatsServiceClient<Channel>,
    info: Arc<Mutex<TrafficInfo>>,
    mut stop_receiver: tokio::sync::mpsc::Receiver<()>,
    tag: &str,
) -> Result<(), Status> {
    while stop_receiver.try_recv().is_err() {
        let (upload, download) = (
            call_api(&mut client, tag, "uplink").await?,
            call_api(&mut client, tag, "downlink").await?,
        );

        {
            let mut info_guard = info.lock().await;
            info_guard.upload = upload;
            info_guard.download = download;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}

impl TrafficInfoUpdater {
    pub fn new(info: Arc<Mutex<TrafficInfo>>) -> Self {
        Self {
            info,
            stop_sender: None,
        }
    }
    pub async fn start(
        &mut self,
        dst: String,
        tag: &'static str,
    ) -> Result<(), tonic::transport::Error> {
        let mut client = StatsServiceClient::connect(dst.clone()).await;
        for _ in 1..10 {
            client = StatsServiceClient::connect(dst.clone()).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            if client.is_ok() {
                break;
            }
        }
        let client = client?;
        let (sender, receiver) = tokio::sync::mpsc::channel(BUFFER_SIZE);
        self.stop_sender = Some(sender);
        tokio::spawn(updater(client, self.info.clone(), receiver, tag));

        Ok(())
    }
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(sender) = self.stop_sender.as_mut() {
            sender.send(()).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Not Started"))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    async fn test_call_api() -> anyhow::Result<()> {
        let mut client = StatsServiceClient::connect("http://127.0.0.1:15490").await?;
        dbg!(call_api(&mut client, "QV2RAY_API_INBOUND", "downlink").await?);
        Ok(())
    }
}
