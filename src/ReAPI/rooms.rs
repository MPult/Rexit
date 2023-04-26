use serde_json::Value;

use super::Bearer;

pub struct Room {
    id: String,
    // Members: Vec<String>,
}

/// Returns a list of Rooms as per [SPEC](https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3joined_rooms)
pub fn get_rooms(bearer: Bearer, debug: bool) -> Vec<Room> {
    let client = super::new_debug_client(debug);

    let resp = client
        .get("https://matrix.redditspace.com/_matrix/client/v3/joined_rooms")
        .header("Authorization", format!("Bearer {}", bearer.token()))
        .send()
        .expect("Failed to send HTTP request; to obtain rooms");

    let body = resp.text().expect("Error parsing body");
    let json: Value = serde_json::from_str(&body).expect("Error parsing Rooms list JSON");
    let rooms = json["joined_rooms"]
        .as_array()
        .expect("Error parsing array");

    info!("Found {} room(s) ", rooms.len());
    println!("rooms: {:#?}", rooms);

    let mut rooms_array: Vec<Room> = Vec::new();

    for room in rooms {
        // For each room create the Room object and apend to rooms_array
        let room_struct = Room {
            id: room.to_string(),
        };

        rooms_array.push(room_struct);
    }

    return rooms_array;
}

#[cfg(test)]
mod tests {
    use crate::ReAPI;

    #[test]
    fn get_rooms() {
        let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
        let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");

        let bearer = ReAPI::login(username, password, true);

        super::get_rooms(bearer, true);
    }
}
