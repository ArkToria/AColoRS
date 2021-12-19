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
