//! This crate provides a easy way of exporting reddit chats into a few formats (including images).
//! This document is intended for developers/contributors, see the [README](https://github.com/MPult/Rexit) for user-centric documention.

use chrono::SecondsFormat::Secs;
use chrono::{TimeZone, Utc};
use console::style;
use inquire::{self, Password, Text};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;
use std::env;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

// import other files
mod export;
use export::decide_export;
mod cli;
use cli::{Cli, Parser};
mod login;
use login::request_login;
mod id_translation;
use id_translation::id_to_displayname;
mod images;

mod macros;

/// Struct for a singular message.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    author: String,
    message: String,
    timestamp: String,
}
/// Struct containing a chat/room.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Chat {
    id: String,
    messages: Vec<Message>,
}
/// Contains all the chats/rooms.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllChats {
    chats: Vec<Chat>,
}

/// Prepares the logger according to the `RUST_LOG` enviornment variable. If none set it is set to `INFO`
/// Then according to the auth flow either username and password are inquired; or just a bearer token.
/// It runs the sync function, and handles the export.
fn main() {
    // Initialize logging
    // If no log level is set => set to info
    match env::var("RUST_LOG") {
        Ok(value) => debug!("Detected loglevel: {value}"),
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
        trace!("Passoword auth flow");

        let username = Text::new("Your Reddit Username")
            .prompt()
            .expect("Error reading username");

        let password = Password::new("Your Reddit Password")
            .without_confirmation()
            .with_display_toggle_enabled()
            .prompt()
            .expect("Error reading password");

        bearer_token = request_login(username.to_owned(), password.to_owned(), args.debug);
    }

    // Handle output folder stuff
    // Deletes out (we append the batches so this is neccesary)

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
    let rooms = list_rooms(bearer_token.clone(), args.debug);

    let mut allChats = AllChats { chats: Vec::new() };

    // Iterate over rooms and request their messages
    for room in rooms {
        let messages = get_messages(bearer_token.clone(), room.as_str().unwrap(), args.debug);
        allChats.chats.push(messages);
    }

    // debug!("{:#?}", { sync.clone() });
    // info!("Found {} Chats", sync.chats.len());
    decide_export(allChats, args);
}


/// Returns list of all rooms that the user is joined to
fn list_rooms(bearer_token: String, debug: bool) -> Vec<serde_json::Value> {
    // Create a Reqwest client
    let client: reqwest::blocking::Client;
    if debug {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(true) // Used in development to trust a proxy
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Error making Reqwest Client");
    } else {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Error making Reqwest Client");
    }

    let resp = client
        .get("https://matrix.redditspace.com/_matrix/client/v3/joined_rooms")
        .header("Authorization", format!("Bearer {}", bearer_token))
        .send()
        .expect("Failed to send HTTP request; to obtain rooms");

    let body = resp.text().expect("Error parsing body");
    let json: Value = serde_json::from_str(&body).expect("Error parsing Rooms list JSON");
    let rooms = json["joined_rooms"]
        .as_array()
        .expect("Error parsing array");

    info!("Found {} room(s) ", rooms.len());
    return rooms.to_vec();
}

/// Returns a Chat struct for this room
fn get_messages(bearer_token: String, room_id: &str, debug: bool) -> Chat {
    info!("Getting messages for room: {room_id}");

    // Create a Reqwest client
    let client: reqwest::blocking::Client;
    if debug {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(true) // Used in development to trust a proxy
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Error making Reqwest Client");
    } else {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Error making Reqwest Client");
    }

    let url = format!("https://matrix.redditspace.com/_matrix/client/r0/rooms/{room_id}/messages?limit=10000&dir=b");
    println!("{url}");
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", bearer_token))
        .send()
        .expect("Failed to send HTTP request; to obtain messages");

    let body = response.text().expect("Error parsing request body");
    let json: Value = serde_json::from_str(&body).expect("Error parsing JSON response");

    // Contains all the messages for this chat
    let mut chat = Chat {
        id: room_id.to_owned(),
        messages: Vec::new(),
    };

    // Loop through the messages within the chunk
    for message in json["chunk"].as_array().unwrap() {
        // Check if it is a text/image
        if message["type"] == "m.room.message" {
            // Parse the unix timestamp and convert to ISO
            let timestamp = message["origin_server_ts"]
                .as_i64()
                .expect("Failed to parse timestamp")
                / 1000;

            let timestamp = Utc
                .timestamp_opt(timestamp, 0)
                .unwrap()
                .to_rfc3339_opts(Secs, true)
                .to_string();

            // If its a image show the MXC url as content
            let message_content: String;
            if message["content"]["msgtype"] == "m.image" {
                message_content = message["content"]["url"].to_string();
            } else {
                message_content = message["content"]["body"].to_string();
            }

            let message_struct = Message {
                author: id_to_displayname(message["senders"].to_string(), debug),
                message: message_content,
                timestamp: timestamp,
            };
            chat.messages.push(message_struct);
        }
    }
    return chat;
}
