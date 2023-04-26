//! Handles the login flow for sendbird
use crate::ReAPI;


/// Performs the login, returns the bearer token
pub fn request_login(username: String, password: String, debug: bool) -> String {
    // Get Reddits bearer token
    let bearer = ReAPI::login(username, password, debug);

    let bearer_str = bearer.token();

    let client = ReAPI::new_debug_client(debug);

    let response = client
        .get("https://s.reddit.com/api/v1/sendbird/me")
        .header("Authorization", format!("Bearer {}", bearer_str))
        .send()
        .expect("Failed to send HTTP request - to login to sendbird");

    let value: serde_json::Value = serde_json::from_str(response.text().unwrap().as_str()).unwrap();
    return value["sb_access_token"].as_str().unwrap().to_string();
}

#[cfg(test)]
mod tests {
    use crate::ReAPI;

    #[test]
    fn sb_request_login() {
        let client =ReAPI::new_client();

        let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
        let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");
    
        let result = super::request_login( username, password, true);
        println!("{result}");   
    }
}
