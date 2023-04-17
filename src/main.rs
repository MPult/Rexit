use chrono::SecondsFormat::Secs;
use chrono::{TimeZone, Utc};
use serde_json::Value;

use serde::{Deserialize, Serialize};
use serde_json::Result;

// import other files
mod export;
use export::decide_export;

// Define structs for the data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    author: String,
    message: String,
    timestamp: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Chat {
    ID: String,
    messages: Vec<Message>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllChats {
    chats: Vec<Chat>,
}

fn main() {
    // First obtain the bearer token securely
    let bearer_token = rpassword::prompt_password("Your Bearer Token: ").unwrap();
    request_sync(bearer_token);

}

fn request_sync(bearer_token: String) {
    const SYNC_ENDPOINT: &str = "https://matrix.redditspace.com/_matrix/client/r0/sync";

    // Create a Reqwest client
    let client = reqwest::blocking::Client::new();

    // Send an HTTP GET request with the bearer token in the "Authorization" header
    let resp = client
        .get(SYNC_ENDPOINT)
        .header("Authorization", format!("Bearer {}", bearer_token))
        .send();

    // Check for errors and print the response body
    match resp {
        Ok(resp) => {
            match resp.text() {
                Ok(body) => {
                    // Sucessful response
                    extract_chats(body);
                }
                Err(err) => {
                    eprintln!("Error reading response body: {}", err);
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}

// Extract the chats from the API response; puts into the AllChats struct
fn extract_chats(response: String) {
    // Parse to JSON
    let json: Value = serde_json::from_str(&response).unwrap_or_else(|err| {
        println!("Error parsing JSON response: {}", err);
        Value::Null
    });

    // Assign AllChat struct to contain the multiple chats

    let mut all_chats = AllChats { chats: Vec::new() };

    // Access the "join" field within the "rooms" field
    if let Some(join) = json["rooms"]["join"].as_object() {
        // Iterate through each room dynamically

        for (room_id, room_data) in join {
            println!("Room: {}", room_id);
            // Event timeline
            let events = &join[room_id]["timeline"]["events"];

            // Assign the struct to contain the messages for this room
            let mut chat = Chat {
                ID: room_id.to_string(),
                messages: Vec::new(),
            };

            // Iterate over the timeline to find events that contain the body key (all messages do; non-message items dont)
            if let Some(events) = events.as_array() {
                for event in events {
                    // Check if it is a message
                    if let Some(content) = event["content"].as_object() {
                        if content.contains_key("body") {
                            // Parse the unix timestamp and convert to ISO
                            let timestamp = event["origin_server_ts"].as_i64().unwrap_or(0) / 1000;
                            let timestamp = Utc.timestamp_opt(timestamp, 0).unwrap();
                            let timestamp = timestamp.to_rfc3339_opts(Secs, true);

                            // Add data to the Message struct
                            let message = Message {
                                author: event["sender"].to_string(),
                                message: event["content"]["body"].to_string(),
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
        println!("'join' field not found within 'rooms'");
    }

    // Call the decide_export function to decide how to export the chats
    decide_export(all_chats);
}
