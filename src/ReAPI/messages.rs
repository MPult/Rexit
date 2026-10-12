use std::collections::HashMap;
use std::path::PathBuf;

use super::{images, Client};
use chrono::{TimeZone, Utc};
use log::{debug, error};
use serde::{Deserialize, Serialize};

/// Struct for a singular message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Matrix event id
    #[serde(default)]
    pub id: String,
    /// Display name of the sender
    pub author: String,
    /// Matrix user id of the sender
    #[serde(default)]
    pub sender: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub content: Content,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Content {
    /// A text message
    Message(String),
    /// An image, as its mxc:// url
    Image(String),
    /// A message that was deleted
    Deleted,
}

impl Content {
    /// Plain text representation, used by the txt and csv exports
    pub fn as_text(&self) -> String {
        match self {
            Content::Message(text) => text.clone(),
            Content::Image(url) => format!("FILE {url}"),
            Content::Deleted => "[deleted]".to_owned(),
        }
    }
}

/// Messages and the room name found in a room's timeline
pub struct Timeline {
    pub messages: Vec<Message>,
    pub name: Option<String>,
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
    #[serde(default)]
    event_id: String,
    sender: String,
    room_id: String,
    content: InternalContent,

    #[serde(rename = "origin_server_ts")]
    timestamp: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct InternalContent {
    msgtype: Option<String>,
    body: Option<String>,
    url: Option<String>,
    info: Option<InternalImageInfo>,
    /// Room name, on m.room.name events
    name: Option<String>,
    #[serde(rename = "m.relates_to")]
    relates_to: Option<InternalRelation>,
    #[serde(rename = "m.new_content")]
    new_content: Option<Box<InternalContent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InternalRelation {
    rel_type: Option<String>,
    event_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InternalImageInfo {
    mimetype: Option<String>,
}

/// Downloads all messages of a room. `members` maps user ids to display names.
pub async fn list_messages(
    client: &Client,
    id: String,
    members: &HashMap<String, String>,
    image_download: bool,
    no_usernames: bool,
    out: PathBuf,
    redact: bool,
) -> Timeline {
    let mut output: Vec<Message> = vec![];
    let mut name: Option<String> = None;
    // Latest edit per original event id
    let mut edits: HashMap<String, String> = HashMap::new();
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
            .await
            .expect("Failed to send HTTP request; to obtain messages");

        // Deserialize response
        let status = response.status();
        let body = response.text().await.unwrap();
        let messages: Result<InternalMessages, serde_json::Error> =
            serde_json::from_str(body.as_str());
        if let Err(e) = messages {
            let snippet: String = body.chars().take(300).collect();
            error!("Failed to parse messages for room {id} (HTTP {status}): {e}; body: {snippet}");
            break;
        }
        let messages = messages.unwrap();
        output.reserve(messages.chunk.len());

        // Iterate over events, newest first
        for message in messages.chunk {
            if message.messages_type == "m.room.name" {
                // Keep the newest name
                if name.is_none() {
                    name = message.content.name;
                }
                continue;
            }
            if message.messages_type != "m.room.message" {
                continue;
            }

            // Edits replace the text of the original message
            if let Some(relation) = &message.content.relates_to {
                if relation.rel_type.as_deref() == Some("m.replace") {
                    if let (Some(original), Some(new_body)) = (
                        relation.event_id.clone(),
                        message.content.new_content.and_then(|c| c.body),
                    ) {
                        edits.entry(original).or_insert(new_body);
                    }
                    continue;
                }
            }

            // Handle the no-usernames CLI flag
            let author = if no_usernames {
                "N/A".to_owned()
            } else if let Some(displayname) = members.get(&message.sender) {
                displayname.clone()
            } else {
                super::get_user(client, message.sender.clone(), redact)
                    .await
                    .displayname
            };

            let content = if let Some(url) = message.content.url {
                // Is a file; if option is set download the image
                if image_download {
                    images::get_image(
                        client,
                        url.clone(),
                        out.clone(),
                        &out.join("messages/images"),
                        redact,
                    )
                    .await;
                }
                Content::Image(url)
            } else if let Some(body) = message.content.body {
                Content::Message(body)
            } else {
                // Redacted events keep their type but lose their content
                Content::Deleted
            };

            output.push(Message {
                id: message.event_id,
                author,
                sender: message.sender,
                timestamp: unix_millis_to_utc(message.timestamp),
                content,
            });
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

    apply_edits(&mut output, &edits);

    // Oldest first
    output.reverse();
    output.sort_by_key(|message| message.timestamp);

    Timeline {
        messages: output,
        name,
    }
}

fn apply_edits(messages: &mut [Message], edits: &HashMap<String, String>) {
    for message in messages {
        if let Some(new_body) = edits.get(&message.id) {
            message.content = Content::Message(new_body.clone());
        }
    }
}

fn unix_millis_to_utc(unix_time: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(unix_time / 1000, 0).unwrap()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Content, Message};
    use chrono::Utc;

    #[test]
    fn content_serialization() {
        // Text keeps the v1 shape so old readers still work
        assert_eq!(
            serde_json::to_string(&Content::Message("hi".into())).unwrap(),
            r#"{"Message":"hi"}"#
        );
        assert_eq!(
            serde_json::to_string(&Content::Image("mxc://reddit.com/abc".into())).unwrap(),
            r#"{"Image":"mxc://reddit.com/abc"}"#
        );
        assert_eq!(
            serde_json::to_string(&Content::Deleted).unwrap(),
            r#""Deleted""#
        );
    }

    #[test]
    fn edits_replace_text() {
        let mut messages = vec![Message {
            id: "$1".into(),
            author: "a".into(),
            sender: "@a".into(),
            timestamp: Utc::now(),
            content: Content::Message("typo".into()),
        }];
        let edits = HashMap::from([("$1".to_string(), "fixed".to_string())]);

        super::apply_edits(&mut messages, &edits);

        assert_eq!(messages[0].content, Content::Message("fixed".into()));
    }
}
