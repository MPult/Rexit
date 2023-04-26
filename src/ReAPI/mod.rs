#![allow(dead_code)]
pub mod images;
pub mod users;

pub type Client = reqwest::blocking::Client;
pub type Bearer = std::string::String;



pub fn new_debug_client(debug: bool) -> Client {
    // Build the client
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

    client
}

pub fn new_client() -> Client {
    let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .build()
            .expect("Error making Reqwest Client"); 

    client
}