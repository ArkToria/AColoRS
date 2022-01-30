use clap::ArgMatches;
use log::debug;
use log::error;
use utils::net::{tcp_get_available_port, tcp_port_is_available};

use std::process;

use crate::args::Args;
use crate::Result;

pub fn serve(args: &Args) -> Result<bool> {
    debug!("Serve with args: {:?}", args);
    let matches = get_serve_matches(args);
    let interface = matches.value_of("interface").unwrap_or("[::1]");
    let mut port = get_port_from(matches);

    test_and_set_port(&mut port);

    match server::serve(interface, port) {
        Ok(()) => Ok(true),
        Err(e) => {
            error!("unravel error: {:?}", &e);
            process::exit(1);
        }
    }
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
    match matches.value_of("port").unwrap_or("19198").parse() {
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
        *port = if let Some(p) = tcp_get_available_port(19198) {
            p
        } else {
            error!("No port avaiable");
            process::exit(1);
        }
    }
}
