//! Converts the given userID into a displayname using the API. ([SPEC](https://spec.matrix.org/v1.6/client-server-api/#get_matrixclientv3profileuseriddisplayname))

use cached::proc_macro::cached;
use serde_json::Value;

/// Converts the userids  into displaynames; obtains data through a API request, uses function cache
#[cached]
pub fn id_to_displayname(id: String, debug: bool) -> String {
    // Create a Reqwest client
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

    let url = format!("https://matrix.redditspace.com/_matrix/client/r0/profile/{id}/displayname");
    // Request name from API
    let response = client
        .get(url)
        .send()
        .expect("Failed to send HTTP request");

    // Parse the json
    let displayname: Value = serde_json::from_str(
        &response
            .text()
            .expect("Error getting Displayname - HTTP Request"),
    )
    .expect("Error getting Displayname - JSON parsing");

    let displayname = displayname["displayname"].as_str().unwrap();

    debug!("Got User lookup: {}, with ID: {}", displayname, id);

    return displayname.to_owned();
}
