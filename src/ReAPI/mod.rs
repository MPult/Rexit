//! Reddit matrix api
#![allow(non_snake_case, dead_code)]

mod images;
mod login;
mod messages;
mod rooms;
pub(crate) mod saved_posts;
pub(crate) mod subreddit;
mod users;

pub use rooms::download_rooms;
pub use rooms::whoami;
pub use rooms::Room;

pub use saved_posts::download_saved_posts;

pub use subreddit::download_subreddit;

pub use messages::Content;
pub use messages::Message;

pub use users::get_user;

/// Get a string field from Reddit JSON, empty if missing
pub(crate) fn json_str(value: &serde_json::Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

pub struct Client {
    reqwest_client: reqwest::Client,
    bearer: Option<String>,
}

impl Client {
    /// GET a path on Reddit's OAuth API using the bearer token
    pub(crate) async fn reddit_api_get(&self, path: &str) -> Option<serde_json::Value> {
        let response = self
            .reqwest_client
            .get(format!("https://oauth.reddit.com/{path}"))
            .header("Authorization", format!("Bearer {}", self.bearer_token()))
            // Reddit's API rules ask for a descriptive user agent
            .header(
                "User-Agent",
                concat!("rexit/", env!("CARGO_PKG_VERSION"), " (by u/mpult)"),
            )
            .send()
            .await
            .expect("Failed to send HTTP request");

        let status = response.status();
        let body = response.text().await.unwrap();
        if !status.is_success() {
            let snippet: String = body.chars().take(300).collect();
            log::error!("Reddit API request to {path} failed (HTTP {status}): {snippet}");
            return None;
        }

        match serde_json::from_str(&body) {
            Ok(json) => Some(json),
            Err(e) => {
                log::error!("Failed to parse Reddit API response from {path}: {e}");
                None
            }
        }
    }
}

pub fn new_client(debug: bool) -> Client {
    // Build the client
    let client = if debug {
        reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(60))
            .danger_accept_invalid_certs(true) // Used in development to trust a proxy
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.5615.121 Safari/537.36")
            .build()
            .expect("Error making Reqwest Client")
    } else {
        reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.5615.121 Safari/537.36")
            .build()
            .expect("Error making Reqwest Client")
    };

    Client {
        reqwest_client: client,
        bearer: None,
    }
}
