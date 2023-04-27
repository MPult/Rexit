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
pub fn get_user(client: &Client, id: String) -> User {
    info!(target: "get_user", "id: {}", id.clone());
    println!("{}", id);
    let url = format!("https://matrix.redditspace.com/_matrix/client/r0/profile/{id}/displayname",);

    let response = client.get(url).send().expect("Failed to send HTTP request");

    let value: serde_json::Value = serde_json::from_str(response.text().unwrap().as_str()).unwrap();

    info!("displayname: {}", value["displayname"].clone());

    User {
        id: id,
        displayname: value["displayname"].as_str().unwrap().to_string(),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn get_user() {
        let client = super::super::new_client(true);
        let id = "@t2_9b09u6gps:reddit.com".to_string();

        let result = super::get_user(&client, id);

        assert_eq!(result.displayname, "rexitTest");
    }
}
