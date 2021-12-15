use clap::{crate_authors, crate_description, crate_version, App};

pub fn app() -> App<'static, 'static> {
    let app = App::new("ACoRus")
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .help_message("Try '--help' for more information");
    app
}

type Arg = clap::Arg<'static, 'static>;

#[derive(Clone)]
pub struct ACArg {
    claparg: Arg,
    pub name: &'static str,
    pub doc_short: &'static str,
    pub doc_long: &'static str,
    pub hidden: bool,
    pub kind: ACArgKind,
}
#[allow(dead_code)]
#[derive(Clone)]
pub enum ACArgKind {
    /// A boolean switch.
    Switch {
        long: &'static str,
        short: Option<&'static str>,
    },
    /// A flag the accepts a single value.
    Flag {
        long: &'static str,
        short: Option<&'static str>,
        value_name: &'static str,
        possible_values: Vec<&'static str>,
    },
}
#[allow(dead_code)]
pub fn all_args_and_flags() -> Vec<ACArg> {
    let mut args = vec![];
    args
}
