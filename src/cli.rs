//! Argument Parser

pub use clap::{Args, Parser, Subcommand};

/// CLI argument parser, see the Cli struct for the possible arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]

/// Struct that contains the possible args
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
    #[arg(short,long)]
    pub images: bool,

    /// What folder to output to
    #[arg(short,long,default_value="./out")]
    pub out: String,
}
