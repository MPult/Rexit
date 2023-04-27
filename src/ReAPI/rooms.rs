use super::Client;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Room {
    pub id: String,
    messages: Option<Vec<super::Message>>,
}

impl Room {
    pub fn from(id: String) -> Room {
        Room { id, messages: None }
    }

    pub fn messages(&mut self, client: &Client) -> Vec<super::Message> {
        if self.messages.is_some() {
            return self.messages.clone().unwrap();
        }

        self.get_messages(client, self.to_owned());
        return self.messages.clone().unwrap();
    }

    fn get_messages(&mut self, client: &Client, room: Room) {
        self.messages = Some(super::messages::list_messages(client, room));
    }
}

/// Returns list of all rooms that the user is joined to as per [SPEC](https://spec.matrix.org/v1.6/client-server-api/#get_matrixclientv3directorylistroomroomid)
pub fn list_rooms(client: &Client) -> Vec<Room> {
    let resp = client
        .reqwest_client
        .get("https://matrix.redditspace.com/_matrix/client/v3/joined_rooms")
        .header("Authorization", format!("Bearer {}", client.bearer_token()))
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

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "creds"]
    fn list_rooms() {
        let (username, password) = get_login();
        let mut client = super::super::new_client(true);

        client.login(username, password);

        let rooms = super::list_rooms(&client);

        println!("{:?}", rooms);
    }

    fn get_login() -> (String, String) {
        let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
        let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");

        (username, password)
    }
}
