pub use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The formats to export to. Options: csv,json,txt
    #[arg(short, long)]
    pub formats: String,

    /// To use the bearer token flow, instead of username and password
    #[arg(short, long)]
    pub token: bool,

    /// Allow debugging of Rexit
    #[arg(long)]
    pub debug: bool,

    /// Output images too (outputs to images folder)
    #[arg(long)]
    pub images: bool,
}
