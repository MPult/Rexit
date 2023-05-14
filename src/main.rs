//! This crate provides a easy way of exporting reddit chats into a few formats (including images).
//! This document is intended for developers/contributors, see the [README](https://github.com/MPult/Rexit) for user-centric documentation.

// extern crate pretty_env_logger;
// #[macro_use]
// extern crate log;
use log::{info, trace, warn};
// use log4rs;

use log::LevelFilter;
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use console::style;
use export::{export_saved_posts, export_subreddit};
use inquire::{self, Password, Text};
use log4rs::filter::threshold::ThresholdFilter;
use std::path::PathBuf;
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
        client = init(debug, token, images, out.clone(), true).await;

        // Creates out folder
        if !out.join("messages").exists() {
            std::fs::create_dir(out.join("messages").clone()).unwrap();
            std::fs::create_dir(out.join("messages/images").clone()).unwrap();
        }

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
        client = init(debug, token, images, out.clone(), true).await;

        // Creates out folder
        if !out.join("saved_posts").exists() {
            std::fs::create_dir(out.join("saved_posts").clone()).unwrap();
            std::fs::create_dir(out.join("saved_posts/images").clone()).unwrap();
        }

        // Gets saved posts
        let saved_posts = ReAPI::download_saved_posts(&client, images);

        let saved_posts = saved_posts.await;

        // Exports messages to files.
        let export_formats: Vec<&str> = formats.split(",").collect();

        // Export Saved posts
        export_saved_posts(saved_posts, export_formats, &out);
    } else if let cli::Commands::Subreddit {
        name,
        formats,
        token,
        images,
        out,
        debug,
    } = args.command
    {
        // Initialize
        client = init(debug, token, images, out.clone(), false).await;

        // Creates out folder
        if !out.join("subreddit").exists() {
            std::fs::create_dir(out.join("subreddit").clone()).unwrap();
            std::fs::create_dir(out.join("subreddit/images").clone()).unwrap();
        }
        // Gets saved posts
        let subreddit = ReAPI::download_subreddit(&client, name, images);

        let subreddit = subreddit.await;

        // Exports messages to files.
        let export_formats: Vec<&str> = formats.split(",").collect();

        // Export Saved posts
        export_subreddit(subreddit, export_formats, &out);
    }
}

/// Handles all the init stuff for rexit
async fn init(debug: bool, token: bool, images: bool, out: PathBuf, auth: bool) -> Client {
    // Create a Client
    let mut client = ReAPI::new_client(debug);

    // Handle the debug stuff
    if debug {
        println!("{}\n{}", 
            style("The --debug flag accepts untrusted HTTPS certificates which can be a potential security risk").red().bold(), 
            style("This option is only recommended if you know what your are doing and you want to debug Rexit").red().bold());
    }

    // Initialize logging
    let level = log::LevelFilter::Info;
    let file_path = "./rexit.log";

    // Build a stderr logger.

    let stderr = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M)(utc)} - {h({l})}: {m}{n}",
        )))
        .target(Target::Stderr)
        .build();

    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M)(utc)} - {h({l})}: {m}{n}",
        )))
        .build(file_path)
        .unwrap();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config);

    // Authenticate if needed
    if auth {
        // Handle the three auth flows
        if token == true {
            // Use the bearer token flow
            trace!("Bearer token auth flow");

            client.login_with_token(
                Password::new("Your Bearer Token")
                    .prompt()
                    .expect("Error reading bearer token"),
            );
        } else if std::env::var("REXIT_USERNAME").is_ok() && std::env::var("REXIT_PASSWORD").is_ok()
        {
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
    }

    // Handle output folder stuff
    if !out.exists() {
        std::fs::create_dir(out.clone()).unwrap();
    }

    return client;
}
