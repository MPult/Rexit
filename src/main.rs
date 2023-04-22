use chrono::SecondsFormat::Secs;
use chrono::{TimeZone, Utc};
use console::style;
use inquire::{self, Password, Text};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::path::PathBuf;

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
// Define structs for the data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    author: String,
    message: String,
    timestamp: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Chat {
    id: String,
    messages: Vec<Message>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllChats {
    chats: Vec<Chat>,
}

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

        bearer_token = request_login(username.to_owned(), password.to_owned());
    }

    // Make sure there is an images folder to output to if images is true
    if args.images && !PathBuf::from("./images").exists() {
        std::fs::create_dir("./images").unwrap();
    }

    // Request the sync which includes the messages in a timeline
    let sync = request_sync(bearer_token, args.debug, args.images).unwrap();

    debug!("{:#?}", { sync.clone() });
    info!("Found {} Chats", sync.chats.len());
    decide_export(sync, args);
}

fn request_sync(bearer_token: String, debug: bool, images: bool) -> Option<AllChats> {
    const SYNC_ENDPOINT: &str = "https://matrix.redditspace.com/_matrix/client/r0/sync";

    // Create a Reqwest client
    let client: reqwest::blocking::Client;
    if debug {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(true) // Used in development to trust a proxy
            .build()
            .expect("Error making Reqwest Client");
    } else {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .build()
            .expect("Error making Reqwest Client");
    }

    // Send an HTTP GET request with the bearer token in the "Authorization" header
    let resp = client
        .get(SYNC_ENDPOINT)
        .header("Authorization", format!("Bearer {}", bearer_token))
        .send()
        .expect("Failed to send HTTP request");

    // Read the response body
    let body = resp.text().expect("Failed to read response body");

    debug!("Sync Response: {:#?}", body);

    // Parse response body to JSON
    let json: Value = serde_json::from_str(&body).expect("Error parsing JSON response");

    // Assign AllChat struct to contain the multiple chats
    let mut all_chats = AllChats { chats: Vec::new() };

    // Access the "join" field within the "rooms" field
    if json["rooms"]["join"].as_object().is_none() {
        error!("rooms.join is none");
        exit!(0);
    }
    let join = json["rooms"]["join"].as_object().unwrap();
    // Iterate through each room dynamically
    for (room_id, _) in join {
        info!("Found a Room: {}", room_id);
        // Event timeline
        let events = &join[room_id]["timeline"]["events"];

        // Assign the struct to contain the messages for this room
        let mut chat = Chat {
            id: room_id.to_string(),
            messages: Vec::new(),
        };

        // Iterate over the timeline to find events that are messages (text or image)
        let events = events.as_array();
        if events.is_none() {
            error!("Events is none");
            exit!(0);
        }
        let events = events.unwrap();
        for event in events {
            // Check if it is a message
            if event["type"] == "m.room.message" {
                // Parse the unix timestamp and convert to ISO
                let timestamp = event["origin_server_ts"]
                    .as_i64()
                    .expect("Failed to parse timestamp")
                    / 1000;

                let timestamp = Utc
                    .timestamp_opt(timestamp, 0)
                    .unwrap()
                    .to_rfc3339_opts(Secs, true)
                    .to_string();

                // Check if message is a image. If yes use the URL as message text.
                // Possible types are: m.text, m.image (matrix specs for more but not implemented in reddit)
                let mut message_text = String::default();

                if event["content"]["msgtype"] == "m.text" {
                    // Text message
                    message_text = event["content"]["body"].as_str()?.to_string()
                }
                if images && event["content"]["msgtype"] == "m.image" {
                    // Image message
                    message_text = event["content"]["url"].as_str()?.to_string();
                    images::export_image(&client, message_text.clone());
                }

                // Translates the userID of the message into a displayname
                let displayname = id_to_displayname(event["sender"].as_str()?.to_string(), debug);

                // Add data to the Message struct
                let message = Message {
                    author: displayname,
                    message: message_text,
                    timestamp: timestamp,
                };

                // Push the individual message into the chats struct
                chat.messages.push(message)
            }
        }

        // Push the chat into the AllChats struct
        all_chats.chats.push(chat)
    }

    // Call the decide_export function to decide how to export the chats
    Some(all_chats)
}
