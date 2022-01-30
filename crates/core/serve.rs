use anyhow::anyhow;
use clap::ArgMatches;
use log::debug;
use log::error;
use utils::net::{tcp_get_available_port, tcp_port_is_available};

use std::net::SocketAddr;
use std::process;

use crate::args::Args;
use crate::Result;

const DEFAULT_PORT: u16 = 19198;

pub fn serve(args: &Args) -> Result<bool> {
    debug!("Serve with args: {:?}", args);

    let matches = get_serve_matches(args);
    let interface = matches.value_of("interface").unwrap_or("[::1]");
    let mut port = get_port_from(matches);

    test_and_set_port(&mut port);

    let address = format!("{}:{}", interface, port);
    let address = address_from_string(&address)?;
    match server::serve(address) {
        Ok(()) => Ok(true),
        Err(e) => {
            error!("unravel error: {:?}", &e);
            process::exit(1);
        }
    }
}

fn address_from_string(address: &String) -> Result<SocketAddr> {
    let result: SocketAddr = match address.parse() {
        Ok(a) => a,
        Err(_) => return Err(anyhow!("Invalid address: {}.", address)),
    };
    Ok(result)
}

fn get_serve_matches(args: &Args) -> &ArgMatches {
    args.matches()
        .subcommand_matches("serve")
        .unwrap_or_else(|| {
            error!("No subcommand \"serve\".");
            process::exit(1);
        })
}

fn get_port_from(matches: &ArgMatches) -> u16 {
    let result = matches
        .value_of("port")
        .map(|p| p.parse())
        .unwrap_or(Ok(DEFAULT_PORT));
    match result {
        Ok(x) => x,
        Err(_) => {
            error!("The port needs to be an integer");
            process::exit(1);
        }
    }
}

fn test_and_set_port(port: &mut u16) {
    if *port != 19198 && !tcp_port_is_available(*port) {
        error!("The port is not available");
        process::exit(1);
    }

    if !tcp_port_is_available(*port) {
        *port = if let Some(p) = tcp_get_available_port(DEFAULT_PORT) {
            p
        } else {
            error!("No port avaiable");
            process::exit(1);
        }
    }
}