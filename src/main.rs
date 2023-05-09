//! This crate provides a easy way of exporting reddit chats into a few formats (including images).
//! This document is intended for developers/contributors, see the [README](https://github.com/MPult/Rexit) for user-centric documentation.

// extern crate pretty_env_logger;
// #[macro_use]
// extern crate log;
use log::{debug, error, info, trace, warn};
use log4rs;

use console::style;
use export::export_saved_posts;
use inquire::{self, Password, Text};
use std::{env, path::PathBuf};
use ReAPI::Client;

// import other files
mod ReAPI;
mod cli;
mod export;
mod macros;

use cli::{Cli, Parser};

#[tokio::main]
async fn main() {
    // Parse the CLI args
    let args = Cli::parse();

    // Create an ReAPI client
    let client: Client;

    // Init the program
    if let cli::Commands::Messages {
        formats,
        token,
        images,
        out,
        debug,
    } = args.command
    {
        // Initialize
        client = init(debug, token, images, out.clone()).await;

        // Get list of rooms
        let rooms = ReAPI::download_rooms(&client, images).await;

        // Exports messages to files.
        let export_formats: Vec<&str> = formats.split(",").collect();

        // Export chats
        for room in rooms {
            for format in export_formats.clone() {
                match format {
                    "txt" => export::export_room_chats_txt(room.to_owned(), &out),
                    "json" => export::export_room_chats_json(room.to_owned(), &out),
                    "csv" => export::export_room_chats_csv(room.to_owned(), &out),
                    _ => println!("Not valid Format"),
                }
            }
        }
    } else if let cli::Commands::Saved {
        formats,
        token,
        images,
        out,
        debug,
    } = args.command
    {
        // Initialize
        client = init(debug, token, images, out.clone()).await;

        // Gets saved posts
        let saved_posts = ReAPI::download_saved_posts(&client, images);

        let saved_posts = saved_posts.await;

        // Exports messages to files.
        let export_formats: Vec<&str> = formats.split(",").collect();

        // Export Saved posts
        export_saved_posts(saved_posts, export_formats, &out);
    }
}

/// Handles all the init stuff for rexit
async fn init(debug: bool, token: bool, images: bool, out: PathBuf) -> Client {
    // Create a Client
    let mut client = ReAPI::new_client(debug);

    // Handle the debug stuff
    if debug {
        println!("{}\n{}", 
            style("The --debug flag accepts untrusted HTTPS certificates which can be a potential security risk").red().bold(), 
            style("This option is only recommended if you know what your are doing and you want to debug Rexit").red().bold());

        // Initialize logging
        log4rs::init_file("./log4rs-debug.yaml", Default::default()).unwrap();
    } else {
        // Initialize logging
        log4rs::init_file("./log4rs.yaml", Default::default()).unwrap();
    }

    // Handle the three auth flows
    if token == true {
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
    if out.exists() {
        std::fs::remove_dir_all(out.clone()).expect("Error deleting out folder");
    }

    // Creates out folders
    std::fs::create_dir(out.clone()).unwrap();
    std::fs::create_dir(out.join("messages")).unwrap();
    std::fs::create_dir(out.join("saved_posts")).unwrap();

    // Make sure there is an images folder to output to if images is true
    if images {
        std::fs::create_dir(out.join("messages/images")).unwrap();
        std::fs::create_dir(out.join("saved_posts/images")).unwrap();
    }

    return client;
}
