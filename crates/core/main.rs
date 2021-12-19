use std::{error, process};

use args::Args;

mod app;
mod args;

type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;
fn main() {
    if let Err(err) = Args::parse().and_then(try_main) {
        eprintln!("{}", err);
        process::exit(2);
    }
}

fn try_main(args: Args) -> Result<()> {
    use args::Command::*;

    let matched = match args.command()? {
        Serve => serve(&args),
        Profile => manage_profiles(&args),
        Plugin => manage_plugins(&args),
    }?;
    if matched {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
fn serve(args: &Args) -> Result<bool> {
    Ok(true)
}
fn manage_profiles(args: &Args) -> Result<bool> {
    Ok(true)
}
fn manage_plugins(args: &Args) -> Result<bool> {
    Ok(true)
}
