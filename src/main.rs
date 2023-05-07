//! This crate provides a easy way of exporting reddit chats into a few formats (including images).
//! This document is intended for developers/contributors, see the [README](https://github.com/MPult/Rexit) for user-centric documentation.

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use console::style;
use export::export_saved_posts;
use inquire::{self, Password, Text};
use std::env;

// import other files
mod ReAPI;
mod cli;
mod export;
mod macros;

use cli::{Cli, Parser};

/// Prepares the logger according to the `RUST_LOG` environment variable. If none set it is set to `INFO`
/// Then according to the auth flow either username and password are inquired; or just a bearer token.
/// It runs the sync function, and handles the export.
#[tokio::main]
async fn main() {
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
    } else if std::env::var("REXIT_USERNAME").is_ok() && std::env::var("REXIT_PASSWORD").is_ok() {
        warn!("Found password and username enviornment variables");

        let username = std::env::var("REXIT_USERNAME").unwrap();
        let password = std::env::var("REXIT_PASSWORD").unwrap();
        client.login(username, password).await;
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

        client.login(username.to_owned(), password.to_owned()).await;
    }

    info!("Login Successful");

    // Handle output folder stuff
    // Deletes the output folder (we append the batches so this is necessary)
    if args.out.exists() {
        std::fs::remove_dir_all(&args.out).expect("Error deleting out folder");
    }

    // Creates out folders
    std::fs::create_dir(&args.out).unwrap();
    std::fs::create_dir(args.out.join("messages")).unwrap();
    std::fs::create_dir(args.out.join("saved_posts")).unwrap();

    // Make sure there is an images folder to output to if images is true
    if args.images {
        std::fs::create_dir(args.out.join("messages/images")).unwrap();
        std::fs::create_dir(args.out.join("saved_posts/images")).unwrap();
    }

    // Get list of rooms
    let rooms = ReAPI::download_rooms(&client, args.images).await;

    // Gets saved posts
    let saved_posts = ReAPI::download_saved_posts(&client, args.images);

    // Export logic
    // Exports messages to files. Add image if its set to args
    let export_formats: Vec<&str> = args.formats.split(",").collect();

    // Export chats
    for room in rooms {
        for format in export_formats.clone() {
            match format {
                "txt" => export::export_room_chats_txt(room.to_owned(), &args.out),
                "json" => export::export_room_chats_json(room.to_owned(), &args.out),
                "csv" => export::export_room_chats_csv(room.to_owned(), &args.out),
                _ => println!("Not valid Format"),
            }
        }
    }

    let saved_posts = saved_posts.await;

    // Export Saved posts
    export_saved_posts(saved_posts, export_formats, &args.out);
}
