use std::io::Write;
use std::{env, ffi::OsString, io, process, sync::Arc};

use clap::ArgMatches;

use crate::app;
use crate::Result;

pub enum Command {
    Serve,
}

#[derive(Clone, Debug)]
pub struct Args(Arc<ArgsImp>);

#[derive(Clone, Debug)]
struct ArgsImp {
    matches: ArgMatches,
}

impl Args {
    pub fn parse() -> Result<Args> {
        let matches = clap_matches(env::args_os())?;

        pretty_env_logger::init();

        Ok(Args(Arc::new(ArgsImp { matches })))
    }
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
impl Args {
    fn matches(&self) -> &ArgMatches {
        &self.0.matches
    }
}

fn clap_matches<I, T>(args: I) -> Result<clap::ArgMatches>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let err = match app::app().try_get_matches_from(args) {
        Ok(matches) => return Ok(matches),
        Err(err) => err,
    };
    if err.use_stderr() {
        return Err(err.into());
    }
    // Explicitly ignore any error returned by write!. The most likely error
    // at this point is a broken pipe error, in which case, we want to ignore
    // it and exit quietly.
    //
    // (This is the point of this helper function. clap's functionality for
    // doing this will panic on a broken pipe error.)
    let _ = write!(io::stdout(), "{}", err);
    process::exit(0);
}
