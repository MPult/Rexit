use cached::proc_macro::cached;

#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
}

#[cached]
pub fn get_user(client: &super::Client, id: String) -> User {
    let url = reqwest::Url::parse(format!("https://matrix.redditspace.com/_matrix/client/r0/profile/{}/displayname", id).as_str()).unwrap();    

    let response = client
        .get(url)
        .send()
        .expect("Failed to send HTTP request");
    
    User {
        id: "jlkdf".to_string(),
        name: "hasdf".to_string()
    }
}

