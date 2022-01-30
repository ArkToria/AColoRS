use std::net::{SocketAddr, SocketAddrV4, TcpListener};
use std::thread;

use anyhow::Result;
use anyhow::{anyhow, Context};
use log::{debug, error, info};
use tonic::transport::Server;

use crate::greeter::acolors_proto::greeter_server::GreeterServer;
use crate::greeter::AColoRSGreeter;

mod greeter;

pub fn serve(interface: &str, interface_port: u16) -> Result<()> {
    debug!(
        "AColoRS Server is starting at {}:{}",
        interface, interface_port
    );
    let address = format!("{}:{}", interface, interface_port);
    let address = address_from_string(&address)?;
    check_tcp_bind(&address)?;

    let t = thread::spawn(move || {
        let addr: SocketAddr = std::net::SocketAddr::V4(address);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Could not build tokio runtime");

        match rt.block_on(start_server(addr)) {
            Ok(()) => {
                info!("gRPC Server stopped normally.");
            }
            Err(e) => {
                error!("gRPC Server error: {}", e);
            }
        };
    });

    t.join().unwrap();

    Ok(())
}

async fn start_server(addr: SocketAddr) -> Result<()> {
    let greeter = AColoRSGreeter::default();

    info!("gRPC server is available at http://{}\n", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await
        .context("Failed to start gRPC server.")?;
    Ok(())
}

fn address_from_string(address: &String) -> Result<SocketAddrV4> {
    let result: SocketAddrV4 = match address.parse() {
        Ok(a) => a,
        Err(_) => return Err(anyhow!("Invalid address: {}.", address)),
    };
    Ok(result)
}

fn check_tcp_bind(bind_address: &SocketAddrV4) -> Result<()> {
    if (TcpListener::bind(&bind_address)).is_err() {
        return Err(anyhow!("Cannot start server on address {}.", bind_address));
    }
    Ok(())
}
