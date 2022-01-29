use std::{error, process};

use args::Args;
use log::debug;

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
    Ok(true)
}
