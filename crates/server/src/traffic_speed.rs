use std::time::Duration;

use core_protobuf::{
    acolors_proto::TrafficInfo,
    v2ray_api_proto::{stats_service_client::StatsServiceClient, GetStatsRequest},
};
use spdlog::info;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio_stream::wrappers::BroadcastStream;
use tonic::{codegen::StdError, transport::Channel, Status};

use crate::BUFFER_SIZE;

#[derive(Debug)]
pub struct TrafficInfoSender {
    sender: broadcast::Sender<TrafficInfo>,
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
async fn producer(
    mut client: StatsServiceClient<Channel>,
    sender: Sender<TrafficInfo>,
    tag: &str,
) -> Result<(), Status> {
    loop {
        let (upload, download) = (
            call_api(&mut client, tag, "uplink").await?,
            call_api(&mut client, tag, "downlink").await?,
        );

        if sender.send(TrafficInfo { upload, download }).is_err() {
            info!("Producer Exited");
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

impl TrafficInfoSender {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(BUFFER_SIZE);

        Self { sender }
    }
    pub fn subscribe(&self) -> Receiver<TrafficInfo> {
        self.sender.subscribe()
    }
    pub async fn start<D>(&self, dst: D, tag: &'static str) -> Result<(), tonic::transport::Error>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let client = StatsServiceClient::connect(dst).await?;
        tokio::spawn(producer(client, self.sender.clone(), tag));
        Ok(())
    }
}

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::StreamExt;

pub struct TrafficStream {
    stream: BroadcastStream<TrafficInfo>,
}
impl TrafficStream {
    pub fn new(receiver: broadcast::Receiver<TrafficInfo>) -> Self {
        let stream = BroadcastStream::new(receiver);
        Self { stream }
    }
}

impl futures::Stream for TrafficStream {
    type Item = Result<TrafficInfo, Status>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut_self = self.get_mut();
        let poll = mut_self.stream.poll_next_unpin(cx);
        if poll.is_pending() && crate::SHUTDOWN.load(std::sync::atomic::Ordering::Relaxed) {
            return Poll::Ready(None);
        }
        let mapped_poll: Poll<Option<Result<TrafficInfo, Status>>> =
            poll.map_err(|error| Status::data_loss(error.to_string()));
        mapped_poll
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
