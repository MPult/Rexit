use std::path::PathBuf;

use super::Client;
use serde::Serialize;
use serde_json::Value;
use log::info;

#[derive(Debug, Clone, Serialize)]
pub struct Room {
    pub id: String,
    pub(crate) messages: Option<Vec<super::Message>>,
}

impl Room {
    async fn download(id: String, client: &Client, image_download: bool, no_usernames: bool, out: PathBuf, redact: bool) -> Room {
        Room {
            id: id.clone(),
            messages: download_messages(&client, id.clone(), image_download, no_usernames, out, redact).await,
        }
    }

    pub fn messages(&self) -> Vec<super::Message> {
        return self.messages.clone().unwrap();
    }
}

async fn download_messages(
    client: &Client,
    id: String,
    image_download: bool,
    no_usernames: bool,
    out: PathBuf,
    redact: bool
) -> Option<Vec<super::Message>> {
    Some(super::messages::list_messages(client, id, image_download, no_usernames, out, redact).await)
}

/// Returns list of all rooms that the user is joined to as per [SPEC](https://spec.matrix.org/v1.6/client-server-api/#get_matrixclientv3directorylistroomroomid)
pub async fn download_rooms(client: &Client, image_download: bool, no_usernames: bool, out: PathBuf, redact: bool) -> Vec<Room> {
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

    // Read rooms from json
    let rooms = json["joined_rooms"]
        .as_array()
        .expect("Error parsing array")
        .to_owned();

    // Move rooms into a Vec<Room>
    let rooms = rooms.iter().map(move |room| {
        Room::download(room.to_string().replace("\"", ""), client, image_download, no_usernames, out.to_owned(), redact)
    });

    let mut rooms_2: Vec<Room> = vec![];
    for room in rooms {
        rooms_2.push(room.await);
    }

    info!("Found {} room(s) ", rooms_2.len());

    return rooms_2;
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[tokio::test]
    #[ignore = "creds"]
    async fn list_rooms() {
        let (username, password) = get_login();
        let mut client = super::super::new_client(true);

        client.login(username, password).await;

        let rooms = super::download_rooms(&client, true, false, PathBuf::from("./out"), false);

        println!("{:?}", rooms.await);
    }

    fn get_login() -> (String, String) {
        let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
        let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");

        (username, password)
    }
}
