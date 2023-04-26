
#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub displayname: String,
}

pub fn get_user(client: &super::Client, id: String) -> User {
    let url = reqwest::Url::parse(format!("https://matrix.redditspace.com/_matrix/client/r0/profile/{}/displayname", id).as_str()).unwrap();    

    let response = client
        .get(url)
        .send()
        .expect("Failed to send HTTP request");

    let value: serde_json::Value = serde_json::from_str(response.text().unwrap().as_str()).unwrap();

    User {
        id: id,
        displayname: value["displayname"].as_str().unwrap().to_string()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn get_user() {
        let client = super::super::new_client();
        let id = "@t2_9b09u6gps:reddit.com".to_string();

        let result = super::get_user(&client, id);

        assert_eq!(result.displayname, "rexitTest");
    }
}