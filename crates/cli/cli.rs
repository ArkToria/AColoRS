use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Serve on the specified port and address
    Serve(ServeArgs),
}

#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Config path
    #[clap(short, long,default_value_t = String::from("./config/acolors.json"))]
    pub config: String,
    /// Interface to bind on
    #[clap(short, long,default_value_t = String::from("127.0.0.1"))]
    pub interface: String,
    /// Database path
    #[clap(short, long,default_value_t = String::from("./config/acolors.db"))]
    pub dbpath: String,
    /// Core path
    #[clap(short = 'k', long,default_value_t = String::from("v2ray"))]
    pub core_path: String,
    /// Core name
    #[clap(short = 't', long,default_value_t = String::from("v2ray"))]
    pub core_name: String,
    /// Port to use [default: available port in [11451..19198)],
    #[clap(short, long)]
    pub port: Option<u16>,
}
