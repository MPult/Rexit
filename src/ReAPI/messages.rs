use super::Client;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Struct for a singular message.
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub author: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub content: Content,
}

#[derive(Debug, Clone, Serialize)]
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
    let mut batch: String = String::new();
    // Loop over the batching
    loop {

        let url = format!(
            "https://matrix.redditspace.com/_matrix/client/r0/rooms/{id}/messages?limit=10000&dir=b&from={batch}");

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

            // Detect if message is text or file
            if message.content.url.is_some() {
                // Is a file
                output.push(Message {
                    author: super::get_user(client, message.sender).displayname,
                    timestamp: unix_millis_to_utc(message.timestamp),
                    content: Content::Image(super::images::get_image(&client, message.content.url.unwrap())),
                })
            } else if message.content.body.is_some() {
                // Text Message
                output.push(Message {
                    author: super::get_user(client, message.sender).displayname,
                    timestamp: unix_millis_to_utc(message.timestamp),
                    content: Content::Message(message.content.body.unwrap()),
                })
            }
        }
        
        // Check for end condition
        if messages.end == "t0_0" {
            debug!("Found messages end");
            break;
        } else {
            // Update new batch variable
            batch = messages.end;
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
    #[ignore]
    fn list_messages() {
        let (username, password) = get_login();

        let mut client = new_client(true);

        client.login(username, password);

        let rooms = super::super::download_rooms(&client);

        let messages =super::list_messages(&client, rooms[1].clone().id);
        println!("{:#?}", messages);
    }

    fn get_login() -> (String, String) {
        let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
        let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");

        (username, password)
    }
}
