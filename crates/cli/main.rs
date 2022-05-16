use std::process;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use spdlog::error;

mod cli;
mod serve;

fn main() {
    if let Err(e) = spdlog::init_env_level() {
        error!("Log Error: {}", e);
    }
    if let Err(err) = try_main(Cli::parse()) {
        println!("{}", err);
        process::exit(2);
    }
}

fn try_main(cli: Cli) -> Result<()> {
    let matched = match cli.command {
        Commands::Serve(serve_args) => serve::serve(serve_args),
    }?;
    if matched {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
