use clap::{crate_authors, crate_description, crate_version, App, Arg};

pub fn app() -> App<'static> {
    let app = App::new("acolors")
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommands(vec![App::new("serve")
            .about("Serve on the specified port and address")
            .args(&[
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .takes_value(true)
                    .help("Config path (default: \"./config/acolors.json\")"),
                Arg::new("interface")
                    .short('i')
                    .long("interface")
                    .takes_value(true)
                    .help("Interface to bind on (default: 127.0.0.1)"),
                Arg::new("dbpath")
                    .short('d')
                    .long("dbpath")
                    .takes_value(true)
                    .help("Database path (default: \"./config/acolors.db\")"),
                Arg::new("corepath")
                    .short('k')
                    .long("corepath")
                    .takes_value(true)
                    .help("Core path (default: \"v2ray\")"),
                Arg::new("port")
                    .short('p')
                    .long("port")
                    .takes_value(true)
                    .help("Which port to use (default: 19198)"),
            ])]);
    app
}
