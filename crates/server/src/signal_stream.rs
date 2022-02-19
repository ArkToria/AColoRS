use std::{
    pin::Pin,
    task::{Context, Poll},
};

use core_protobuf::acolors_proto::ProfileSignal;
use futures::StreamExt;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tonic::Status;

pub struct SignalStream {
    stream: BroadcastStream<profile_manager::ProfileSignal>,
}
impl SignalStream {
    pub fn new(receiver: broadcast::Receiver<profile_manager::ProfileSignal>) -> Self {
        let stream = BroadcastStream::new(receiver);
        return Self { stream };
    }
}

impl futures::Stream for SignalStream {
    type Item = Result<ProfileSignal, Status>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let poll = self.get_mut().stream.poll_next_unpin(cx);
        let map_poll: Poll<Option<Result<core_protobuf::acolors_proto::ProfileSignal, Status>>> =
            poll.map_ok(|profile_signal| profile_signal.into())
                .map_err(|error| Status::data_loss(error.to_string()));
        map_poll
    }
}
