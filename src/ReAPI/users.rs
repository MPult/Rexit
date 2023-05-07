use super::Client;
use cached::SizedCache;

#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub displayname: String,
}

#[cached::proc_macro::cached(
    type = "SizedCache<String, User>",
    create = "{ SizedCache::with_size(10_000) }",
    convert = r#"{ format!("{}", id) }"#
)]
pub async fn get_user(client: &Client, id: String) -> User {
    let url = format!("https://matrix.redditspace.com/_matrix/client/r0/profile/{id}/displayname",);

    let response = client
        .reqwest_client
        .get(url)
        .send()
        .await
        .expect("Failed to send HTTP request");

    let value: serde_json::Value =
        serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();

    debug!("Found user: {}", value["displayname"].clone());

    User {
        id: id,
        displayname: value["displayname"].as_str().unwrap().to_string(),
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn get_user() {
        let client = super::super::new_client(true);
        let id = "@t2_9b09u6gps:reddit.com".to_string();

        let result = super::get_user(&client, id);

        assert_eq!(result.await.displayname, "rexitTest");
    }
}
