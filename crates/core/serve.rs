use anyhow::anyhow;
use clap::ArgMatches;
use spdlog::{debug, error, info};
use utils::net::{tcp_get_available_port, tcp_port_is_available};

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process;

use crate::args::Args;
use crate::Result;

pub fn serve(args: &Args) -> Result<bool> {
    debug!("Serve with args: {:?}", args);

    let matches = get_serve_matches(args);
    let interface = matches.value_of("interface").unwrap_or("127.0.0.1");
    let mut port = get_port_from(matches);
    let database_path = value_of_or(matches, "dbpath", "./config/acolors.db");
    let config_path = value_of_or(matches, "config", "./config/acolors.json");
    let core_path = value_of_or(matches, "corepath", "v2ray");
    let core_name = matches.value_of("corename").unwrap_or("v2ray");

    print_file_path(&database_path, &config_path, &core_path);

    test_and_set_port(&mut port);

    let address = format!("{}:{}", interface, port);
    let address = address_from_string(&address)?;
    match server::serve(address, database_path, core_path, core_name, config_path) {
        Ok(()) => Ok(true),
        Err(e) => {
            error!("unravel error: {:?}", &e);
            process::exit(1);
        }
    }
}

fn value_of_or(matches: &ArgMatches, value: &str, default_path: &str) -> PathBuf {
    let path = matches
        .value_of(value)
        .map_or_else(|| PathBuf::from(default_path), PathBuf::from);

    path
}

fn print_file_path(database_path: &Path, config_path: &Path, core_path: &Path) {
    info!(
        "Database Path: {}",
        database_path.as_os_str().to_string_lossy(),
    );
    info!(
        "Configuration File Path: {}",
        config_path.as_os_str().to_string_lossy(),
    );
    info!("Core Path: {}", core_path.as_os_str().to_string_lossy(),);
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
    let result = matches.value_of("port").map(|p| p.parse()).unwrap_or(Ok(0));
    match result {
        Ok(x) => x,
        Err(_) => {
            error!("The port needs to be an integer between 1-65535");
            process::exit(1);
        }
    }
}

fn test_and_set_port(port: &mut u16) {
    if *port == 0 {
        *port = if let Some(p) = tcp_get_available_port(11451..19198) {
            p
        } else {
            error!("No port avaiable");
            process::exit(1);
        }
    } else if !tcp_port_is_available(*port) {
        error!("The port {} is not available", *port);
        process::exit(1);
    }
}
