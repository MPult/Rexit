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
mod ReAPI;
mod cli;
mod macros;

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

    // Create an ReAPI client
    let mut client = ReAPI::new_client(args.debug);

    if args.debug {
        println!("{}\n{}", 
            style("The --debug flag accepts untrusted HTTPS certificates which can be a potential security risk").red().bold(), 
            style("This option is only recommended if you know what your are doing and you want to debug Rexit").red().bold());
    }

    // Decide what auth flow to use
    if args.token == true {
        // Use the bearer token flow
        trace!("Bearer token auth flow");

        client.login_with_token(
            Password::new("Your Bearer Token")
                .prompt()
                .expect("Error reading bearer token"),
        );
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

        client.login(username.to_owned(), password.to_owned());
    }

    // Handle output folder stuff
    // Deletes ./out (we append the batches so this is necessary)
    if PathBuf::from("./out").exists() {
        std::fs::remove_dir_all("./out").expect("Error deleting out folder");
    }

    // Creates out folder
    std::fs::create_dir("./out").unwrap();

    // Make sure there is an images folder to output to if images is true
    if args.images {
        std::fs::create_dir("./out/images").unwrap();
    }

    // Get list of rooms
    let rooms = ReAPI::download_rooms(&client);

    // Export
    for room in rooms {
        export::export_room_chats(room.to_owned());
    }
}
