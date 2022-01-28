use std::ffi::OsStr;
use std::{env, ffi::OsString, io, path::PathBuf, process, sync::Arc};
use std::io::Write;

use pretty_env_logger::env_logger::Logger;

use crate::Result;
use crate::app;

pub enum Command {
    Serve,
    Profile,
    Plugin,
}

#[derive(Clone, Debug)]
pub struct Args(Arc<ArgsImp>);

#[derive(Clone, Debug)]
struct ArgsImp {
    matches: ArgMatches,
}

impl Args {
    pub fn parse() -> Result<Args> {
        let matches = ArgMatches::new(clap_matches(env::args_os())?);

        pretty_env_logger::init();


        matches.to_args()
    }
}

#[derive(Clone, Debug)]
struct ArgMatches(clap::ArgMatches<'static>);

impl Args {
    pub fn command(&self) -> Result<Command> {
        // TODO: need to be implement
        panic!("No command implementation");
    }
}

impl ArgMatches {
    fn new(clap_matches: clap::ArgMatches<'static>) -> ArgMatches {
        ArgMatches(clap_matches)
    }

    fn to_args(self) -> Result<Args> {
        Ok(Args(Arc::new(ArgsImp { 
            matches: self,
        })))
    }

}

impl ArgMatches {
    fn is_present(&self, name:&str) -> bool {
        self.0.is_present(name)
    }

    fn occurrences_of(&self, name: &str) -> u64 {
        self.0.occurrences_of(name)
    }

    fn value_of_lossy(&self, name:&str) -> Option<String> {
        self.0.value_of_lossy(name).map(|s| s.into_owned())
    }

    fn values_of_lossy(&self, name:&str) -> Option<Vec<String>> {
        self.0.values_of_lossy(name)
    }

    fn value_of_os(&self, name:&str) -> Option<&OsStr> {
        self.0.value_of_os(name)
    }

    fn values_of_os(&self, name: &str) -> Option<clap::OsValues<'_>> {
        self.0.values_of_os(name)
    }
}

/// Returns a clap matches object if the given arguments parse successfully.
///
/// Otherwise, if an error occurred, then it is returned unless the error
/// corresponds to a `--help` or `--version` request. In which case, the
/// corresponding output is printed and the current process is exited
/// successfully.
fn clap_matches<I, T>(args: I) -> Result<clap::ArgMatches<'static>>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let err = match app::app().get_matches_from_safe(args) {
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