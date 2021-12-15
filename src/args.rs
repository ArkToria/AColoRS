use std::{path::PathBuf, sync::Arc};

use crate::Result;

#[derive(Clone, Debug)]
struct ArgMatches(clap::ArgMatches<'static>);

#[derive(Clone, Debug)]
pub struct Args(Arc<ArgsImp>);

#[derive(Clone, Debug)]
struct ArgsImp {
    matches: ArgMatches,
    patterns: Vec<String>,
    paths: Vec<PathBuf>,
    using_default_path: bool,
}

impl Args {
    pub fn parse() -> Result<Args> {
        Args
    }
}
