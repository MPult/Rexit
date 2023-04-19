use regex::Regex;
use urlencoding::encode;

// Request a Bearer token using the username and password
pub fn request_login(username: String, password: String) -> String {
    // URL encode the password & username
    let encoded_password: String;
    let username = encode(&username);

    // Reddit is doing a wierd thing where * is not urlencoded. Sorry for everyone that has * and %2A in their password
    if password.contains("*") {
        println!("Password has *");
        encoded_password = password.replace("%2A", "*");
    } else {
        encoded_password = encode(&password).into_owned();
    }

    // Obtain the CSRF token
    let client = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Error making Reqwest Client");

    // Send an HTTP GET request to get the CSRF token
    let resp = client
        .get("https://www.reddit.com/login/")
        .send()
        .expect("Failed to send HTTP request; to obtain CSRF token");

    println!("{:?}", resp);
    let body = resp.text();
    let body = body.expect("Failed to read response body");

    // Regex to find the CSRF token in the body of the HTML
    let csrf =
        Regex::new(r#"<input\s+type="hidden"\s+name="csrf_token"\s+value="([^"]*)""#).unwrap();

    // For the love of god do not touch this code ever; i made a deal with the devel to make this work
    let mut csrf_token: String = String::default();
    for i in csrf.captures_iter(body.as_str()) {
        for i in i.get(1).iter() {
            csrf_token = String::from(i.as_str().clone());
            println!("CSRF Token: {}", csrf_token);
        }
    }



    // Form data for actual login
    let form_data = format!(
        "csrf_token={}&otp=&password={}&dest=https%3A%2F%2Fwww.reddit.com&username={}",
        csrf_token, encoded_password, username
    );

    println!("{:#?}", form_data);

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
        .header("Referer","https://www.reddit.com/login/")
        .header("Accept-Encoding", "gzip, deflate")
        .header("Accept-Language", "en-GB,en-US;q=0.9,en;q=0.8")
        .body(form_data)
        .send()
        .expect("Failed to send HTTP request; to obtain session token");

    println!("{:#?}", response);

    // Request / to get the bearer token
    let response = client 
      .get("https://www.reddit.com/")
      .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.5615.121 Safari/537.36")
      .header("Accept-Encoding", "gzip, deflate")
      .header("Accept-Language", "en-GB,en-US;q=0.9,en;q=0.8")
      .header("Referer","https://www.reddit.com/login/")
      .header("Sec-Fetch-Dest", "document")
      .header("Sec-Fetch-Mode", "navigate")
      .header("Sec-Fetch-Site", "same-origin")
      .header("Sec-Fetch-User", "?1")
      .header("Te", "trailers")
      .send()
      .expect("Error getting bearer token");
  
    // println!("{:#?}", response.text());
    
    // Extract the Bearer Token from the JSON response
    let bearer_regex = Regex::new(r#"accessToken":"([^"]+)"#).unwrap();

    let mut bearer_token: String = String::default();
    for i in bearer_regex.captures_iter(&response.text().unwrap()) {
        for i in i.get(1).iter() {
          bearer_token = String::from(i.as_str().clone());
            println!("Bearer Token: {}", bearer_token);
        }
    }

    return bearer_token;
}
