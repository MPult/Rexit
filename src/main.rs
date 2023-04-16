use chrono::SecondsFormat::Secs;
use chrono::{TimeZone, Utc};
use serde_json::Value;

// Define structs for the data structure
#[derive(Debug)]
struct Message {
    author: String,
    message: String,
    timestamp: String,
}
#[derive(Debug)]
struct Chat {
    ID: String,
    messages: Vec<Message>,
}
#[derive(Debug)]
struct AllChats {
    chats: Vec<Chat>,
}

fn main() {
    // First obtain the bearer token securely
    let bearer_token = rpassword::prompt_password("Your Bearer Token: ").unwrap();
    request_sync(bearer_token)
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

// Extract the chats from the API response
fn extract_chats(response: String) -> Option<Message> {
    // Parse to JSON
    let json: Value = serde_json::from_str(&response).unwrap_or_else(|err| {
        eprintln!("Error parsing JSON response: {}", err);
        Value::Null
    });

    // Access the "join" field within the "rooms" field
    if let Some(join) = json["rooms"]["join"].as_object() {
        // Iterate through each room dynamically

        // Create a lovely house for all the Chat structs to live in; before they are ad
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

                            // Print to stdout with formating
                            println!(
                                "[{}] {}: {}",
                                timestamp, event["sender"], event["content"]["body"]
                            );

                            // Add data to the Message struct
                            let message = Message {
                                author: event["sender"].to_string(),
                                message: event["content"]["body"].to_string(),
                                timestamp: timestamp,
                            };

                            // Push to the Chats struct
                            chat.messages.push(message)
                        }
                    }
                }
            }

            println!("{:#?}", chat);
        }
    } else {
        println!("'join' field not found within 'rooms'");
    }
    None
}
