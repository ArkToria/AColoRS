use std::{error, process};

use args::Args;
use log::debug;
use log::error;
use utils::net::{tcp_get_available_port, tcp_port_is_available};

mod app;
mod args;

type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;
fn main() {
    pretty_env_logger::init();

    if let Err(err) = Args::parse().and_then(try_main) {
        eprintln!("{}", err);
        process::exit(2);
    }
}

fn try_main(args: Args) -> Result<()> {
    use args::Command::*;

    let matched = match args.command()? {
        Serve => serve(&args),
    }?;
    if matched {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
fn serve(args: &Args) -> Result<bool> {
    debug!("Serve with args: {:?}", args);
    let matches = args.matches();
    let interface = matches.value_of("interface").unwrap_or("[::1]");
    let mut port: u16 = match matches.value_of("port").unwrap_or("19198").parse() {
        Ok(x) => x,
        Err(_) => {
            error!("The port needs to be an integer");
            process::exit(1);
        }
    };
    if port != 19198 && !tcp_port_is_available(port) {
        error!("The port is not available");
        process::exit(1);
    }

    if !tcp_port_is_available(port) {
        port = if let Some(p) = tcp_get_available_port(19198) {
            p
        } else {
            error!("No port avaiable");
            process::exit(1);
        }
    }
    match server::serve(interface, port) {
        Ok(()) => Ok(true),
        Err(e) => {
            error!("unravel error: {:?}", &e);
            process::exit(1);
        }
    }
}
