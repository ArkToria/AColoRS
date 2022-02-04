use std::process;

use anyhow::Result;
use args::Args;
use spdlog::error;

mod app;
mod args;
mod serve;

fn main() {
    spdlog::init_env_level().unwrap();
    if let Err(err) = Args::parse().and_then(try_main) {
        error!("{}", err);
        process::exit(2);
    }
}

fn try_main(args: Args) -> Result<()> {
    use args::Command::*;

    let matched = match args.command()? {
        Serve => serve::serve(&args),
    }?;
    if matched {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
