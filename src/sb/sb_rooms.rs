use reqwest::{self, blocking::Client};

pub struct room {
    id: String,
    // Members: Vec<String>,
}

pub fn get_rooms(sb_token: String, user_id: String, token: String, debug: bool) {
    let url = format!("https://sendbirdproxyk8s.chat.redditmedia.com/v3/users/{user_id}/my_group_channels?token={token}&limit=100");

    let client: reqwest::blocking::Client;
    if debug {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(true) // Used in development to trust a proxy
            .build()
            .expect("Error making Reqwest Client");
    } else {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .build()
            .expect("Error making Reqwest Client");
    }

    let response = client
        .get(url)
        .header("Session-Key", sb_token)
        .send()
        .expect("Failed to send HTTP request; to obtain rooms");
    println!("{:#?}", response.text());
    panic!()
}

#[cfg(test)]
mod tests {
    use crate::ReAPI;

    #[test]
    fn sb_get_rooms() {
        let client = ReAPI::new_client();

        let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
        let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");

        let result = super::request_login(&client, username, password, true);
        println!("{result}");
    }
}
