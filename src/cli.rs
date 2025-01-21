//! Argument Parser

use std::path::PathBuf;

pub use clap::{Parser, Subcommand};

/// CLI argument parser, see the Cli struct for the possible arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    // Command Line Options structure
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Messages {
        /// The formats to export to. Options: csv,json,txt
        #[arg(short, long, default_value = "txt,json,csv")]
        formats: String,

        /// To use the bearer token flow, instead of username and password
        #[arg(short, long)]
        token: bool,

        /// Output images too (outputs to images folder)
        #[arg(short, long)]
        images: bool,

        /// What folder to output to
        #[arg(short, long, default_value = "./out")]
        out: PathBuf,

        /// Trust proxy certificates
        #[arg(short, long)]
        debug: bool,

        /// Not Retrieve usernames (Is a lot faster)
        #[arg(long)]
        noUsernames: bool,

        /// Redact
        #[arg(long)]
        redact: bool
    },
    Saved {
        /// The formats to export to. Options: csv,json,txt
        #[arg(short, long, default_value = "txt,json,csv")]
        formats: String,

        /// To use the bearer token flow, instead of username and password
        #[arg(short, long)]
        token: bool,

        /// Output images too (outputs to images folder)
        #[arg(short, long)]
        images: bool,

        /// What folder to output to
        #[arg(short, long, default_value = "./out")]
        out: PathBuf,

        // Trust proxy certificates
        #[arg(short, long)]
        debug: bool,

        /// Not Retrieve usernames (Is a lot faster)
        #[arg(long)]
        noUsernames: bool,
      
        /// Redact
        #[arg(long)]
        redact: bool
    },
    Subreddit {
        /// Name of the subreddit (Example: r/redditdev)
        name: String,

        /// The formats to export to. Options: csv,json,txt
        #[arg(short, long, default_value = "txt,json,csv")]
        formats: String,

        /// To use the bearer token flow, instead of username and password
        #[arg(short, long)]
        token: bool,

        /// Output images too (outputs to images folder)
        #[arg(short, long)]
        images: bool,

        /// What folder to output to
        #[arg(short, long, default_value = "./out")]
        out: PathBuf,

        // Trust proxy certificates
        #[arg(short, long)]
        debug: bool,

        /// Not Retrieve usernames (Is a lot faster)
        #[arg(long)]
        noUsernames: bool,

        /// Redact
        #[arg(long)]
        redact: bool
    },
}
