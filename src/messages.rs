use crate::ReAPI;
use chrono::SecondsFormat::Secs;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Struct containing a chat/room.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chat {
    pub id: String,
    pub messages: Vec<ReAPI::Message>,
    pub next_batch: String,
}

/// Contains all the chats/rooms.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllChats {
    pub chats: Vec<Chat>,
}

/// Returns a Chat struct for this room as per [SPEC](https://spec.matrix.org/v1.6/client-server-api/#get_matrixclientv3roomsroomidmessages)
pub fn get_messages(
    bearer_token: String,
    room_id: &str,
    since: String,
    debug: bool,
    export_images: bool,
) -> Chat {
    info!("Getting messages for room: {room_id}");

    // Create a Reqwest client
    let client = ReAPI::new_client(debug);

    let url;

    // If it is a next batch then add the since
    if since == "REXIT-INITIAL".to_owned() {
        url = format!("https://matrix.redditspace.com/_matrix/client/r0/rooms/{room_id}/messages?limit=10000&dir=b");
    } else {
        url =format!("https://matrix.redditspace.com/_matrix/client/r0/rooms/{room_id}/messages?limit=10000&dir=b&from={since}");
    }

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
        next_batch: String::new(),
    };

    // Loop through the messages within the chunk
    for message in json["chunk"].as_array().unwrap() {
        // Check if it is a text/image
        if message["type"] == "m.room.message" {
            println!("message: {}", message);
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
            if export_images && message["content"]["msgtype"] == "m.image" {
                message_content = message["content"]["url"].as_str().unwrap().to_string();
                ReAPI::get_image(&client, message_content.clone());
            } else {
                let tmp = message["content"]["body"].as_str();
                if tmp.is_none() {
                    warn!("Failed to get message - may have been deleted");
                    continue;
                }
                message_content = tmp.unwrap().to_string();

                let message_struct = ReAPI::Message {
                    author: ReAPI::get_user(&client, message["sender"].to_string().replace("\"", "")).displayname,
                    message: message_content,
                    timestamp: timestamp,
                };
                chat.messages.push(message_struct);
            }
        }
    }
    // Append next batch to chat
    debug!("End token {}", json["end"].as_str().unwrap().to_string());
    chat.next_batch = json["end"].as_str().unwrap().to_string();
    return chat;
}

/// Iterate over all rooms to return chats
pub fn iter_rooms(
    rooms: Vec<ReAPI::Room>,
    bearer: String,
    debug: bool,
    export_images: bool,
) -> AllChats {
    let mut all_chats = AllChats { chats: Vec::new() };

    // Iterate over rooms and request their messages
    for room in rooms {
        let mut next_batch: String = "REXIT-INITIAL".to_owned();

        while next_batch != "t0_0" {
            let mut found_chat = false;
            let chat_struct =
                get_messages(bearer.clone(), &room.id, next_batch, debug, export_images);
            next_batch = chat_struct.next_batch.clone();

            // Check if a chat with that ID already exits; if yes then append the messages
            for chat in all_chats.chats.iter_mut() {
                if chat.id == chat_struct.id {
                    println!("Chat.id is same as chat_struct ID");
                    chat.messages.extend_from_slice(&chat_struct.messages);
                    found_chat = true;
                    break;
                }
            }

            // If the chat is not already present, add it to the list of all chats
            if !found_chat {
                all_chats.chats.push(chat_struct.clone());
            }
        }
    }

    all_chats
}
