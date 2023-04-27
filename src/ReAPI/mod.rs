//! Reddit matrix api
#![allow(non_snake_case, dead_code)]

mod images;
mod login;
mod messages;
mod rooms;
mod users;

pub use images::Image;

pub use rooms::download_rooms;
pub use rooms::Room;

pub use messages::Content;
pub use messages::Message;

pub use users::get_user;
pub use users::User;

pub struct Client {
    reqwest_client: reqwest::blocking::Client,
    bearer: Option<String>,
}

pub fn new_client(debug: bool) -> Client {
    // Build the client
    let client: reqwest::blocking::Client;
    if debug {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(60))
            .danger_accept_invalid_certs(true) // Used in development to trust a proxy
            .build()
            .expect("Error making Reqwest Client");
    } else {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Error making Reqwest Client");
    }

    Client {
        reqwest_client: client,
        bearer: None,
    }
}
