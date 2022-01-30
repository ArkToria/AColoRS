use std::{env, ffi::OsString, sync::Arc};

use clap::ArgMatches;

use crate::app;
use crate::Context;
use crate::Result;

pub enum Command {
    Serve,
}

#[derive(Clone, Debug)]
pub struct Args(Arc<ArgMatches>);

impl Args {
    pub fn parse() -> Result<Args> {
        let matches = clap_matches(env::args_os())?;

        Ok(Args(Arc::new(matches)))
    }

    pub fn matches(&self) -> &ArgMatches {
        &self.0
    }
}

fn clap_matches<I, T>(args: I) -> Result<clap::ArgMatches>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    app::app()
        .try_get_matches_from(args)
        .context("Failed to get clap matches")
}

impl Args {
    pub fn command(&self) -> Result<Command> {
        let subcommand_option = self.matches().subcommand();
        if let Some(subcommand) = subcommand_option {
            match subcommand {
                ("serve", _) => {
                    return Ok(Command::Serve);
                }
                (command, _) => {
                    panic!("No \"{}\" implementation", command);
                }
            };
        };
        unreachable!()
    }
}
