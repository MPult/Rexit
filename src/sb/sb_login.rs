//! Handles the login flow for sendbird
use crate::ReAPI;


/// Performs the login, returns the bearer token
pub fn request_login(client: &ReAPI::Client, username: String, password: String, debug: bool) -> String {
    // Get Reddits bearer token
    println!("{}", ReAPI::login::login(username, password, debug));
}

#[cfg(test)]
mod tests {
    use crate::ReAPI;


    #[test]
    fn sb_request_login() {
        let client =ReAPI::new_client();

        let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
        let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");
    
        let result = super::request_login(&client, username, password, true);

        assert_eq!(result.displayname, "rexitTest");
    }
}
