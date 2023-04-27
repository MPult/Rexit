use super::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Room {
    pub id: String,
}

impl Room {
    pub fn from(id: String) -> Room {
        Room { id }
    }
}

/// Struct for a singular message.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub author: String,
    pub message: String,
    pub timestamp: String,
}

/// Returns list of all rooms that the user is joined to as per [SPEC](https://spec.matrix.org/v1.6/client-server-api/#get_matrixclientv3directorylistroomroomid)
pub fn list_rooms(client: &Client, bearer_token: super::Bearer) -> Vec<Room> {
    let resp = client
        .get("https://matrix.redditspace.com/_matrix/client/v3/joined_rooms")
        .header("Authorization", format!("Bearer {}", bearer_token.token()))
        .send()
        .expect("Failed to send HTTP request; to obtain rooms");

    // Parse json
    let json: Value =
        serde_json::from_str(&resp.text().unwrap()).expect("Error parsing Rooms list JSON");

    // Read rooms from json
    let rooms = json["joined_rooms"]
        .as_array()
        .expect("Error parsing array")
        .to_owned();

    // Move rooms into a Vec<Room>
    let rooms: Vec<Room> = rooms
        .iter()
        .map(|room| Room::from(room.to_string().replace("\"", "")))
        .collect();

    info!("Found {} room(s) ", rooms.len());

    return rooms;
}

pub fn list_messages(_client: &Client) {
    
} // -> Vec<Message>

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "creds"]
    fn list_rooms() {
        let (username, password) = get_login();
        let client = super::super::new_client(true);

        let bearer = super::super::login(&client, username, password);

        let rooms = super::list_rooms(&client, bearer);

        println!("{:?}", rooms);
    }

    fn get_login() -> (String, String) {
        let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
        let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");

        (username, password)
    }
}
