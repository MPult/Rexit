pub mod images;

pub struct Client {
    ReqwestClient: reqwest::blocking::Client,
}

impl Client {
    pub fn new_debug(debug: bool) -> Client {
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
    
        Client { ReqwestClient: client }
    }
    
    pub fn new() -> Client {
        let client = reqwest::blocking::Client::builder()
                .cookie_store(true)
                .build()
                .expect("Error making Reqwest Client"); 
    
        Client { ReqwestClient: client }
    }
}
