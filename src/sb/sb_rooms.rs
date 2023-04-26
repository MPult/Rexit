use reqwest::{self, blocking::Client};

use crate::ReAPI;

pub struct room {
    id: String,
    // Members: Vec<String>,
}

pub fn get_rooms(sb_token: String, user_id: String, token: String, debug: bool) {
    let url = format!("https://sendbirdproxyk8s.chat.redditmedia.com/v3/users/{user_id}/my_group_channels?token={token}&limit=100");

    let client = ReAPI::new_debug_client(debug);

    let response = client
        .get(url)
        .header("session-key", sb_token)
        .send()
        .expect("Failed to send HTTP request; to obtain rooms");
    println!("{:#?}", response.text());
    panic!()
}

#[cfg(test)]
mod tests {
    use crate::sb;

    #[test]
    fn sb_get_rooms() {
        let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
        let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");

        let sb_token = sb::sb_login::request_login(username, password, true);
        let result = sb::sb_rooms::get_rooms(sb_token, "t2_9b09u6gps".to_owned(), "".to_owned(), true);
        println!("{:#?}", result);
    }
}
