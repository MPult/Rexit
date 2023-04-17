pub use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The formats to export to. Options: csv,json,txt
    #[arg(short, long)]
    pub formats: String,
}
