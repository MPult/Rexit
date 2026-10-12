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

        /// Use the bearer token flow (now the default; kept for compatibility)
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
        no_usernames: bool,

        /// Redact
        #[arg(long)]
        redact: bool,

        /// Open the chat viewer after exporting
        #[arg(long)]
        view: bool,
    },
    /// Browse an existing chat export in your web browser
    View {
        /// The export folder to read
        #[arg(short, long, default_value = "./out")]
        out: PathBuf,

        /// Port to serve on (default: a random free port)
        #[arg(short, long, default_value_t = 0)]
        port: u16,

        /// Don't open the browser automatically
        #[arg(long)]
        no_open: bool,
    },
    Saved {
        /// The formats to export to. Options: csv,json,txt
        #[arg(short, long, default_value = "txt,json,csv")]
        formats: String,

        /// Use the bearer token flow (now the default; kept for compatibility)
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
        no_usernames: bool,

        /// Redact
        #[arg(long)]
        redact: bool,
    },
    Subreddit {
        /// Name of the subreddit (Example: r/redditdev)
        name: String,

        /// The formats to export to. Options: csv,json,txt
        #[arg(short, long, default_value = "txt,json,csv")]
        formats: String,

        /// Use the bearer token flow (now the default; kept for compatibility)
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
        no_usernames: bool,

        /// Redact
        #[arg(long)]
        redact: bool,
    },
}
