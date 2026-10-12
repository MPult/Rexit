use std::collections::HashMap;
use std::path::PathBuf;

use super::Client;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    /// Explicit room name; DMs usually have none
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub members: Vec<Member>,
    pub(crate) messages: Option<Vec<super::Message>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub displayname: String,
}

impl Room {
    async fn download(
        id: String,
        client: &Client,
        image_download: bool,
        no_usernames: bool,
        out: PathBuf,
        redact: bool,
    ) -> Room {
        let members = if no_usernames {
            vec![]
        } else {
            download_members(client, &id).await
        };
        let names: HashMap<String, String> = members
            .iter()
            .map(|m| (m.id.clone(), m.displayname.clone()))
            .collect();

        let timeline = super::messages::list_messages(
            client,
            id.clone(),
            &names,
            image_download,
            no_usernames,
            out,
            redact,
        )
        .await;

        Room {
            id,
            name: timeline.name,
            members,
            messages: Some(timeline.messages),
        }
    }

    pub fn messages(&self) -> Vec<super::Message> {
        self.messages.clone().unwrap()
    }

    /// Short id used for the export file names
    pub fn file_stem(&self) -> String {
        self.id[1..10].to_owned()
    }
}

/// Returns the members of a room as per [SPEC](https://spec.matrix.org/v1.6/client-server-api/#get_matrixclientv3roomsroomidmembers)
async fn download_members(client: &Client, id: &str) -> Vec<Member> {
    let url = format!("https://matrix.redditspace.com/_matrix/client/v3/rooms/{id}/members");
    let resp = client
        .reqwest_client
        .get(url)
        .header("Authorization", format!("Bearer {}", client.bearer_token()))
        .send()
        .await
        .expect("Failed to send HTTP request; to obtain room members");

    let json: Value = match serde_json::from_str(&resp.text().await.unwrap_or_default()) {
        Ok(json) => json,
        Err(e) => {
            warn!("Could not read members of room {id}: {e}");
            return vec![];
        }
    };

    json["chunk"]
        .as_array()
        .map(|events| {
            events
                .iter()
                .map(|event| Member {
                    id: super::json_str(&event["state_key"]),
                    displayname: super::json_str(&event["content"]["displayname"]),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Returns the Matrix user id of the logged in user
pub async fn whoami(client: &Client) -> Option<String> {
    let resp = client
        .reqwest_client
        .get("https://matrix.redditspace.com/_matrix/client/v3/account/whoami")
        .header("Authorization", format!("Bearer {}", client.bearer_token()))
        .send()
        .await
        .ok()?;
    let json: Value = serde_json::from_str(&resp.text().await.ok()?).ok()?;
    json["user_id"].as_str().map(str::to_owned)
}

/// Returns list of all rooms that the user is joined to as per [SPEC](https://spec.matrix.org/v1.6/client-server-api/#get_matrixclientv3directorylistroomroomid)
pub async fn download_rooms(
    client: &Client,
    image_download: bool,
    no_usernames: bool,
    out: PathBuf,
    redact: bool,
) -> Vec<Room> {
    let resp = client
        .reqwest_client
        .get("https://matrix.redditspace.com/_matrix/client/v3/joined_rooms")
        .header("Authorization", format!("Bearer {}", client.bearer_token()))
        .send()
        .await
        .expect("Failed to send HTTP request; to obtain rooms");

    // Parse json
    let json: Value =
        serde_json::from_str(&resp.text().await.unwrap()).expect("Error parsing Rooms list JSON");

    debug!("downloadRoomsResponse: {:?}", json);

    // Read rooms from json
    let rooms = json["joined_rooms"]
        .as_array()
        .expect("Error parsing array; likely error with login")
        .to_owned();

    let mut output: Vec<Room> = vec![];
    for room in rooms {
        output.push(
            Room::download(
                super::json_str(&room),
                client,
                image_download,
                no_usernames,
                out.to_owned(),
                redact,
            )
            .await,
        );
    }

    info!("Found {} room(s) ", output.len());

    output
}
