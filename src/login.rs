//! Module to perform the (insanely intricate) login process.
//! 1. GET `reddit.com/login` to obtain the CSRF token to give to the login.
//! 2. POST `reddit.com/login` to login providing username, CSRF token, Password.
//! 3. GET `reddit.com/` to obtain bearer token from the body of response.
//! 4. Perform matrix chat login à la [spec](https://spec.matrix.org/v1.6/client-server-api/#login)
use regex::Regex;
use urlencoding::encode;

/// Performs the login ritual.
pub fn request_login(username: String, password: String, debug: bool) -> String {
    // URL encode the password & username
    let encoded_password: String;
    let username = encode(&username);

    // Reddit is doing a weird thing where * is not urlencoded. Sorry for everyone that has * and %2A in their password
    if password.contains("*") {
        debug!("Password has *; URL-encode was rewritten");
        encoded_password = password.replace("%2A", "*");
    } else {
        encoded_password = encode(&password).into_owned();
    }

    // Obtain the CSRF token
    let client: reqwest::blocking::Client;
    if debug {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(true) // Used in development to trust a proxy
            .build()
            .expect("Error making Reqwest Client");
    } else {
        client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .build()
            .expect("Error making Reqwest Client");
    }

    // Send an HTTP GET request to get the CSRF token
    let resp = client
        .get("https://www.reddit.com/login/")
        .send()
        .expect("Failed to send HTTP request; to obtain CSRF token");

    debug!("CSRF Request Response: {:?}", resp);
    let body = resp.text();
    let body = body.expect("Failed to read response body");

    // Regex to find the CSRF token in the body of the HTML
    let csrf =
        Regex::new(r#"<input\s+type="hidden"\s+name="csrf_token"\s+value="([^"]*)""#).unwrap();

    // For the love of god do not touch this code ever; i made a deal with the devil to make this work
    let mut csrf_token: String = String::default();
    for i in csrf.captures_iter(body.as_str()) {
        for i in i.get(1).iter() {
            csrf_token = String::from(i.as_str().clone());
            debug!("CSRF Token: {}", csrf_token);
        }
    }

    // Form data for actual login
    let form_data = format!(
        "csrf_token={}&otp=&password={}&dest=https%3A%2F%2Fwww.reddit.com&username={}",
        csrf_token, encoded_password, username
    );

    // Perform the actual login post request
    let response = client
        .post("https://www.reddit.com/login")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Sec-Ch-Ua", "\"Not:A-Brand\";v=\"99\", \"Chromium\";v=\"112\"")
        .header("Sec-Ch-Ua-Platform", "Windows")
        .header("Sec-Ch-Ua-Mobile", "?0")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.5615.121 Safari/537.36")
        .header("Origin", "https://www.reddit.com")
        .header("Sec-Fetch-Site", "same-origin")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Dest", "empty")
        .header("Referrer","https://www.reddit.com/login/")
        .header("Accept-Encoding", "gzip, deflate")
        .header("Accept-Language", "en-GB,en-US;q=0.9,en;q=0.8")
        .body(form_data)
        .send()
        .expect("Failed to send HTTP request; to obtain session token");

    debug!("Login Request response: {:#?}", response);

    // Request / to get the bearer token
    let response = client
        .get("https://www.reddit.com/")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.5615.121 Safari/537.36")
        .header("Accept-Encoding", "gzip, deflate")
        .header("Accept-Language", "en-GB,en-US;q=0.9,en;q=0.8")
        .header("Referrer","https://www.reddit.com/login/")
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Site", "same-origin")
        .header("Sec-Fetch-User", "?1")
        .header("Te", "trailers")
        .send()
        .expect("Error getting bearer token");

    // Extract the Bearer Token from the JSON response
    let bearer_regex = Regex::new(r#"accessToken":"([^"]+)"#).unwrap();

    let mut bearer_token: String = String::default();
    for i in bearer_regex.captures_iter(&response.text().unwrap()) {
        for i in i.get(1).iter() {
            bearer_token = String::from(i.as_str().clone());
            debug!("Bearer Token: {}", bearer_token.trim());
        }
    }

    // Login to matrix.reddit.com using the bearer for reddit.com
    let data = format!(
        "{{\"type\":\"com.reddit.token\",\"token\":\"{bearer_token}\",\"initial_device_display_name\":\"Reddit Web Client\"}}",

    );

    debug!("Matrix request body: {:#?}", data);

    let response = client
        .post("https://matrix.redditspace.com/_matrix/client/r0/login")
        .header("Content-Type", "application/json")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.5615.121 Safari/537.36")
        .header("Accept", "application/json")
        .header("Origin", "https://chat.reddit.com")
        .header("Sec-Fetch-Site", "cross-site")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Dest", "empty")
        .header("Accept-Encoding", "gzip, deflate")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Te", "trailers")
        .body(data)
        .send()
        .expect("Failed to send HTTP request; to login to matrix");

    debug!("Matrix login response: {:#?}", response);
    if !response.status().is_success() {
        println!("{}", response.status().as_u16());
        panic!("login failed");
    }

    return bearer_token;
}
