//! Reddit matrix api
#![allow(non_snake_case, dead_code)]

mod images;
mod login;
mod messages;
mod rooms;
mod saved_posts;
mod users;

pub use images::Image;

pub use rooms::download_rooms;
pub use rooms::Room;

pub use saved_posts::download_saved_posts;
pub use saved_posts::Post;

pub use messages::Content;
pub use messages::Message;

pub use users::get_user;
pub use users::User;

pub struct Client {
    reqwest_client: reqwest::Client,
    bearer: Option<String>,
}

pub fn new_client(debug: bool) -> Client {
    // Build the client
    let client: reqwest::Client;
    if debug {
        client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(60))
            .danger_accept_invalid_certs(true) // Used in development to trust a proxy
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.5615.121 Safari/537.36")
            .build()
            .expect("Error making Reqwest Client");
    } else {
        client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.5615.121 Safari/537.36")
            .build()
            .expect("Error making Reqwest Client");
    }

    Client {
        reqwest_client: client,
        bearer: None,
    }
}
