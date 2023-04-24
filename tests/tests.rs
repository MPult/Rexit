#[test]
#[ignore = "creds"]
fn login() {
    let (username, password) = get_login();

    rexit::login(username, password);
}

fn get_login() -> (String, String) {
    let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
    let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");

    (username, password)
}