use std::process;

use anyhow::Result;
use args::Args;

mod app;
mod args;
mod serve;

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
        Serve => serve::serve(&args),
    }?;
    if matched {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
