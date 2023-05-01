use std::vec;

use serde::{Serialize, Deserialize};
use serde_json::Value;

use super::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedList {
    posts: Vec<Post>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Post {
    title: String,
    subreddit_name: String,
    permalink: String,
}

pub fn download_saved_posts(client: &Client) {
    let response = client
        .reqwest_client
        .get("https://www.reddit.com/user/RexitTest/saved.json")
        .send()
        .expect("Failed to send HTTP request");

    let saved_posts: Value = serde_json::from_str(response.text().unwrap().as_str()).unwrap();

    let mut saved_list: Vec<Post> = Vec::<Post>::new();

    // Iterates over all saved posts in the response array
    for post in saved_posts["data"]["children"].as_array().unwrap() {
        let post = Post {
            title: post["data"]["title"].as_str().unwrap().to_string(),
            subreddit_name: post["data"]["subreddit_name_prefixed"].as_str().unwrap().to_string(),
            permalink: post["data"]["permalink"].as_str().unwrap().to_string(),
        };

        saved_list.push(post);
    }

    return saved_list;
}
