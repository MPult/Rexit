//! This crate provides a easy way of exporting reddit chats into a few formats (including images).
//! This document is intended for developers/contributors, see the [README](https://github.com/MPult/Rexit) for user-centric documentation.

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use console::style;
use inquire::{self, Password, Text};
use std::env;
use std::path::PathBuf;

// import other files
mod export;
use export::decide_export;
mod cli;
mod login;
mod id_translation;
mod images;
mod messages;
mod macros;
mod ReAPI;

use cli::{Cli, Parser};

/// Prepares the logger according to the `RUST_LOG` environment variable. If none set it is set to `INFO`
/// Then according to the auth flow either username and password are inquired; or just a bearer token.
/// It runs the sync function, and handles the export.
fn main() {
    // Initialize logging
    // If no log level is set => set to info
    match env::var("RUST_LOG") {
        Ok(value) => debug!("Detected log level: {value}"),
        Err(_) => env::set_var("RUST_LOG", "INFO"),
    }

    pretty_env_logger::init();

    // Parse the CLI args
    let args = Cli::parse();

    if args.debug {
        println!("{}\n{}", 
            style("The --debug flag accepts untrusted HTTPS certificates which can be a potential security risk").red().bold(), 
            style("This option is only recommended if you know what your are doing and you want to debug Rexit").red().bold());
    }

    // Decide what auth flow to use
    let bearer_token: String;
    if args.token == true {
        // Use the bearer token flow
        trace!("Bearer token auth flow");

        bearer_token = Password::new("Your Bearer Token")
            .prompt()
            .expect("Error reading bearer token");
    } else {
        // Use the username password auth flow
        trace!("Password auth flow");

        let username = Text::new("Your Reddit Username")
            .prompt()
            .expect("Error reading username");

        let password = Password::new("Your Reddit Password")
            .without_confirmation()
            .with_display_toggle_enabled()
            .prompt()
            .expect("Error reading password");

        bearer_token = login::request_login(username.to_owned(), password.to_owned(), args.debug);
    }

    // Handle output folder stuff
    // Deletes ./out (we append the batches so this is necessary)

    if PathBuf::from("./out").exists() {
        std::fs::remove_dir_all("./out").expect("Error deleting out folder");
    }

    // Creates out folder
    std::fs::create_dir("./out").unwrap();

    // Make sure there is an images folder to output to if images is true
    if args.images && !PathBuf::from("./out/images").exists() {
        std::fs::create_dir("./out/images").unwrap();
    }

    // Get list of rooms
    let rooms = messages::list_rooms(bearer_token.clone(), args.debug);

    let all_chats = messages::iter_rooms(rooms, bearer_token, args.debug, args.images);

    decide_export(all_chats, args);
}
