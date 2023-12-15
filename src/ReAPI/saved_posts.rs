use std::path::PathBuf;

use super::{images, Client};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedList {
    posts: Vec<SavedPost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPost {
    pub title: String,
    pub subreddit_name: String,
    pub permalink: String,
    pub img_url: Vec<String>,
    pub body_text: String,
}

pub async fn download_saved_posts(client: &Client, image_download: bool, out: PathBuf, redact: bool) -> Vec<SavedPost> {
    info!("Getting Saved Posts");

    let mut after_token = String::new();
    let mut saved_list: Vec<SavedPost> = Vec::<SavedPost>::new();

    loop {
        let url = format!("https://www.reddit.com/saved.json?after={after_token}");

        let response = client
            .reqwest_client
            .get(url)
            .send()
            .await
            .expect("Failed to send HTTP request");

        let saved_posts: Result<Value, _> =
            serde_json::from_str(response.text().await.unwrap().as_str());
        if saved_posts.is_err() {
            return vec![];
        }
        let saved_posts = saved_posts.unwrap();

        // Iterates over all saved posts in the response array
        for post in saved_posts["data"]["children"].as_array().unwrap() {
            // Get all image urls
            let mut images = Vec::<String>::new();

            // If post has images
            if !post["data"]["preview"].is_null() {
                for image in post["data"]["preview"]["images"].as_array().unwrap() {
                    // The preview URL is HTML encoded (&amp; etc) so we need to decode it
                    let url = image["source"]["url"].as_str().unwrap().to_string();
                    let url = html_escape::decode_html_entities(&url);

                    if image_download {
                        images::get_image(
                            &client,
                            url.to_string(),
                            out.clone(),
                            &std::path::PathBuf::from("./out/saved_posts/images"),
                            redact,
                        )
                        .await;
                    }

                    images.push(url.to_string())
                }
            }

            // Link posts require extra massaging to make work
            if !post["data"]["link_title"].is_null() {
                let post = SavedPost {
                    title: post["data"]["link_title"].as_str().unwrap().to_string(),
                    subreddit_name: post["data"]["subreddit_name_prefixed"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                    permalink: post["data"]["permalink"].as_str().unwrap().to_string(),
                    img_url: images,
                    body_text: post["data"]["selftext"].as_str().unwrap().to_string(),
                };

                saved_list.push(post);
            } else {
                // Normal text post
                let post = SavedPost {
                    title: post["data"]["title"].as_str().unwrap().to_string(),
                    subreddit_name: post["data"]["subreddit_name_prefixed"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                    permalink: post["data"]["permalink"].as_str().unwrap().to_string(),
                    img_url: images,
                    body_text: post["data"]["selftext"].as_str().unwrap().to_string(),
                };

                saved_list.push(post);
            }
        }
        if saved_posts["data"]["after"] == json!(null) {
            break;
        }

        after_token = saved_posts["data"]["after"].as_str().unwrap().to_string();
    }

    return saved_list;
}
