use anyhow::anyhow;
use clap::ArgMatches;
use spdlog::{debug, error};
use utils::net::{tcp_get_available_port, tcp_port_is_available};

use std::net::SocketAddr;
use std::process;

use crate::args::Args;
use crate::Result;

const DEFAULT_PORT: u16 = 19198;

pub fn serve(args: &Args) -> Result<bool> {
    debug!("Serve with args: {:?}", args);

    let matches = get_serve_matches(args);
    let interface = matches.value_of("interface").unwrap_or("127.0.0.1");
    let mut port = get_port_from(matches);
    let database_path = matches.value_of("dbpath").unwrap_or("").to_string();

    test_and_set_port(&mut port);

    let address = format!("{}:{}", interface, port);
    let address = address_from_string(&address)?;
    match server::serve(address, database_path) {
        Ok(()) => Ok(true),
        Err(e) => {
            error!("unravel error: {:?}", &e);
            process::exit(1);
        }
    }
}

fn address_from_string(address: &str) -> Result<SocketAddr> {
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
            error!("The port needs to be an integer between 1-65535");
            process::exit(1);
        }
    }
}

fn test_and_set_port(port: &mut u16) {
    let port_not_available = !tcp_port_is_available(*port);
    if *port != DEFAULT_PORT && port_not_available {
        error!("The port is not available");
        process::exit(1);
    }

    if port_not_available {
        *port = if let Some(p) = tcp_get_available_port(11451..19198) {
            p
        } else {
            error!("No port avaiable");
            process::exit(1);
        }
    }
}
