use std::{
    pin::Pin,
    task::{Context, Poll},
};

use core_protobuf::acolors_proto::AColorSignal;
use futures::StreamExt;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tonic::Status;

pub struct SignalStream {
    stream: BroadcastStream<profile_manager::AColorSignal>,
    closed: bool,
}
impl SignalStream {
    pub fn new(receiver: broadcast::Receiver<profile_manager::AColorSignal>) -> Self {
        let stream = BroadcastStream::new(receiver);
        Self {
            stream,
            closed: false,
        }
    }
}

impl futures::Stream for SignalStream {
    type Item = Result<AColorSignal, Status>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.closed {
            return Poll::Ready(None);
        }
        let mut_self = self.get_mut();
        let poll = mut_self.stream.poll_next_unpin(cx);
        let mapped_poll: Poll<Option<Result<core_protobuf::acolors_proto::AColorSignal, Status>>> =
            poll.map_ok(|profile_signal| {
                match profile_signal {
                    acolors_signal::AColorSignal::Shutdown => {
                        mut_self.closed = true;
                        acolors_signal::AColorSignal::Shutdown
                    }
                    signal => signal,
                }
                .into()
            })
            .map_err(|error| Status::data_loss(error.to_string()));
        mapped_poll
    }
}
