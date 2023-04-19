use chrono::SecondsFormat::Secs;
use chrono::{TimeZone, Utc};
use inquire::{self, Password, Text};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::exit;

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
    pretty_env_logger::init();

    // Parse the CLI args
    let args = Cli::parse();

    // Decide what auth flow to use
    let bearer_token: String;
    if args.token == true {
        // Use the bearer token flow
        trace!("Bearer token auth flow");

        bearer_token = Password::new("Your Bearer Token")
            .prompt()
            .expect("Error reading bearer token"); //rpassword::prompt_password("Your Bearer Token: ")
    } else {
        // Use the username password auth flow
        trace!("Passoword auth flow");

        let username = Text::new("Your Reddit Username")
            .prompt()
            .expect("Error reading username"); //username.trim().to_string();

        let password = Password::new("Your Reddit Password")
            .without_confirmation()
            .with_display_toggle_enabled()
            .prompt()
            .expect("Error reading password"); //rpassword::prompt_password("Your Password: ")

        bearer_token = request_login(username.to_owned(), password.to_owned());
    }

    // Request the sync which includes the messages in a timeline
    let sync = request_sync(bearer_token).unwrap();

    debug!("{:#?}", { sync.clone() });
    info!("Found {} Chats", sync.chats.len());
    decide_export(sync, args);
}

fn request_sync(bearer_token: String) -> Option<AllChats> {
    const SYNC_ENDPOINT: &str = "https://matrix.redditspace.com/_matrix/client/r0/sync";

    // Create a Reqwest client
    let client = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Error making Reqwest Client");

    debug!("Bearer Token: {}", bearer_token);

    // Send an HTTP GET request with the bearer token in the "Authorization" header
    let resp = client
        .get(SYNC_ENDPOINT)
        .header("Authorization", format!("Bearer {}", bearer_token))
        .send()
        .expect("Failed to send HTTP request");

    // Read the response body
    let body = resp.text().expect("Failed to read response body");

    // Parse response body to JSON
    let json: Value = serde_json::from_str(&body).expect("Error parsing JSON response");

    // Assign AllChat struct to contain the multiple chats
    let mut all_chats = AllChats { chats: Vec::new() };

    // Access the "join" field within the "rooms" field
    if let Some(join) = json["rooms"]["join"].as_object() {
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

            // Iterate over the timeline to find events that contain the body key (all messages do; non-message items dont)
            if let Some(events) = events.as_array() {
                for event in events {
                    // Check if it is a message
                    if let Some(content) = event["content"].as_object() {
                        if content.contains_key("body") {
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

                            // Add data to the Message struct
                            let message = Message {
                                author: event["sender"].as_str()?.to_string(),
                                message: event["content"]["body"].as_str()?.to_string(),
                                timestamp: timestamp,
                            };

                            // Push the individual message into the chats struct
                            chat.messages.push(message)
                        }
                    }
                }
            }
            // Push the chat into the AllChats struct
            all_chats.chats.push(chat)
        }
    } else {
        error!("Something went wrong - Check Token/Password");
        exit(0);
    }

    // Call the decide_export function to decide how to export the chats
    Some(all_chats)
}
