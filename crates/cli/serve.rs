use anyhow::anyhow;
use spdlog::{debug, error, info};
use utils::net::{tcp_get_available_port, tcp_port_is_available};

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process;

use crate::cli::ServeArgs;
use crate::Result;

pub fn serve(args: ServeArgs) -> Result<bool> {
    debug!("Serve with args: {:?}", args);

    let interface = args.interface;
    let port = test_and_get_port(&args.port)?;
    let database_path = PathBuf::from(args.dbpath);
    let config_path = PathBuf::from(args.config);
    let core_path = PathBuf::from(args.core_path);
    let core_name = args.core_name;

    print_file_path(&database_path, &config_path, &core_path);

    let address = format!("{}:{}", interface, port);
    let address = address_from_string(&address)?;
    match server::serve(address, database_path, core_path, &core_name, config_path) {
        Ok(()) => Ok(true),
        Err(e) => {
            error!("unravel error: {:?}", &e);
            process::exit(1);
        }
    }
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

fn test_and_get_port(port: &Option<u16>) -> Result<u16> {
    match port {
        Some(p) => {
            if !tcp_port_is_available(*p) {
                Err(anyhow!("The port {} is not available", *p))
            } else {
                Ok(*p)
            }
        }
        None => {
            if let Some(p) = tcp_get_available_port(11451..19198) {
                Ok(p)
            } else {
                Err(anyhow!("No port avaiable"))
            }
        }
    }
}
