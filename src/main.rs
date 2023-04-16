use std::ptr::null;

use serde_json::Value;

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
                    extract_chats(body)
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
fn extract_chats(response: String) {
    // Parse to JSON
    let json: Value = serde_json::from_str(&response).unwrap_or_else(|err| {
        eprintln!("Error parsing JSON response: {}", err);
        Value::Null
    });

    // Access the "join" field within the "rooms" field
    if let Some(join) = json["rooms"]["join"].as_object() {
        // Iterate through each room dynamically
        for (room_id, room_data) in join {
            println!("Room: {}", room_id);
            // Event timeline
            let events = &join[room_id]["timeline"]["events"];

            // Iterate over the timeline to find events that contain the body key (all messages do; non-message items dont)
            if let Some(events) = events.as_array() {
                for event in events {

                    // Check if it is a message
                    if let Some(content) = event["content"].as_object() {
                        if content.contains_key("body") {

                            // Print to stdout with formating
                            println!("[{}] {}: {}", event["origin_server_ts"], event["sender"], event["content"]["body"])
                        }
                    }
                }
            }
        }
    } else {
        println!("'join' field not found within 'rooms'");
    }
}
