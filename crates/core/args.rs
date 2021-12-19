use std::{env, ffi::OsString, io, path::PathBuf, process, sync::Arc};

use pretty_env_logger::env_logger::Logger;

use crate::Result;

pub enum Command {
    Serve,
    Profile,
    Plugin,
}

#[derive(Clone, Debug)]
pub struct Args(Arc<ArgsImp>);

#[derive(Clone, Debug)]
struct ArgsImp {
    /// Mid-to-low level routines for extracting CLI arguments.
    matches: ArgMatches,
    /// The patterns provided at the command line and/or via the -f/--file
    /// flag. This may be empty.
    patterns: Vec<String>,
    /// A matcher built from the patterns.
    ///
    /// It's important that this is only built once, since building this goes
    /// through regex compilation and various types of analyses. That is, if
    /// you need many of theses (one per thread, for example), it is better to
    /// build it once and then clone it.
    //matcher: PatternMatcher,
    /// The paths provided at the command line. This is guaranteed to be
    /// non-empty. (If no paths are provided, then a default path is created.)
    paths: Vec<PathBuf>,
    /// Returns true if and only if `paths` had to be populated with a single
    /// default path.
    using_default_path: bool,
}

impl Args {
    pub fn parse() -> Result<Args> {
        panic!("No Args parse() implementation");
    }
}

#[derive(Clone, Debug)]
struct ArgMatches(clap::ArgMatches<'static>);

impl Args {
    pub fn command(&self) -> Result<Command> {
        panic!("no Args.command(&self) implementation")
    }
}
