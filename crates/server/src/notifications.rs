use std::pin::Pin;

use crate::signal_stream::SignalStream;
use core_protobuf::acolors_proto::{notifications_server::Notifications, *};
use spdlog::debug;
use tokio::sync::broadcast;
use tonic::{Request, Response, Status};

type Stream<T> = Pin<Box<dyn futures::Stream<Item = Result<T, Status>> + Send>>;

pub struct AColoRSNotifications {
    sender: broadcast::Sender<profile_manager::AColorSignal>,
}
impl AColoRSNotifications {
    pub fn new(sender: broadcast::Sender<profile_manager::AColorSignal>) -> Self {
        Self { sender }
    }
}

#[tonic::async_trait]
impl Notifications for AColoRSNotifications {
    type GetNotificationsStream = Stream<AColorSignal>;
    async fn get_notifications(
        &self,
        request: Request<GetNotificationsRequest>,
    ) -> Result<Response<Self::GetNotificationsStream>, Status> {
        debug!("Client connected from {:?}", request.remote_addr());

        Ok(Response::new(
            Box::pin(SignalStream::new(self.sender.subscribe())) as Self::GetNotificationsStream,
        ))
    }
}
