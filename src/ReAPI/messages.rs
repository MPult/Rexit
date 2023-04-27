use super::Client;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Struct for a singular message.
#[derive(Debug, Clone)]
pub struct Message {
    pub author: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub content: Content,
}

#[derive(Debug, Clone)]
pub enum Content {
    Image(super::Image),
    Message(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InternalMessages {
    start: String,
    end: String,
    chunk: Vec<InternalMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InternalMessage {
    #[serde(rename = "type")]
    messages_type: String,
    sender: String,
    room_id: String,
    content: InternalContent,

    #[serde(rename = "origin_server_ts")]
    timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InternalContent {
    body: Option<String>,
    url: Option<String>,
    info: Option<InternalImageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InternalImageInfo {
    mimetype: String,
}

pub fn list_messages(client: &Client, id: String) -> Vec<Message> {
    let mut output: Vec<Message> = vec![];

    let url = format!(
        "https://matrix.redditspace.com/_matrix/client/r0/rooms/{}/messages?limit=10000&dir=b",
        id
    );

    // Send request to get messages
    let response = client
        .reqwest_client
        .get(url)
        .header("Authorization", format!("Bearer {}", client.bearer_token()))
        .send()
        .expect("Failed to send HTTP request; to obtain messages");

    // Deserialize response
    let messages: Result<InternalMessages, serde_json::Error> =
        serde_json::from_str(response.text().unwrap().as_str());
    let messages = messages.unwrap();
    output.reserve(messages.chunk.len());

    // Iterate over messages
    for message in messages.chunk {
        if let Some(text) = message.content.body {
            output.push(Message {
                author: message.sender,
                timestamp: unix_millis_to_utc(message.timestamp),
                content: Content::Message(text),
            })
        } else if let Some(image_url) = message.content.url {
            output.push(Message {
                author: message.sender,
                timestamp: unix_millis_to_utc(message.timestamp),
                content: Content::Image(super::images::get_image(&client, image_url)),
            })
        }
    }

    return output;
}

fn unix_millis_to_utc(unix_time: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(unix_time / 1000, 0).unwrap()
}

#[cfg(test)]
mod tests {
    use super::super::new_client;

    #[test]
    fn list_messages() {
        let (username, password) = get_login();

        let mut client = new_client(true);

        client.login(username, password);

        let rooms = super::super::download_rooms(&client);

        super::list_messages(&client, rooms[0].clone().id);

        panic!();
    }

    fn get_login() -> (String, String) {
        let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
        let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");

        (username, password)
    }
}
