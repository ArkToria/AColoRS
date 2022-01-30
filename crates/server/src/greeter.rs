use log::info;
use tonic::{Request, Response, Status};

use acolors_proto::greeter_server::Greeter;
use acolors_proto::PingReply;
use acolors_proto::PingRequest;

pub mod acolors_proto {
    tonic::include_proto!("acolors");
}
#[derive(Default)]
pub struct AColoRSGreeter {}

#[tonic::async_trait]
impl Greeter for AColoRSGreeter {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingReply>, Status> {
        info!("Got a request from {:?}", request.remote_addr());

        let reply = acolors_proto::PingReply {
            message: format!("Received Ping from {}.", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}
