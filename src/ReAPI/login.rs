use console::style;
use log::debug;
use regex::Regex;

impl super::Client {
    pub fn logged_in(&self) -> bool {
        self.bearer.is_some()
    }

    pub fn bearer_token(&self) -> String {
        if let Some(token) = self.bearer.clone() {
            return token.clone();
        }

        println!("{}", style("You are not logged in").red().bold());
        crate::exit!(0);
    }

    pub fn login_with_token(&mut self, bearer: String) {
        self.bearer = Some(bearer);
    }

    /// Log into Reddit returning the Bearer
    pub async fn login(&mut self, username: String, password: String) {
        // URL encode the password & username
        let encoded_password: String;
        let username = urlencoding::encode(&username);

        // Reddit is doing a weird thing where * is not urlencoded. Sorry for everyone that has * and %2A in their password
        if password.contains("*") {
            debug!("Password has *; URL-encode was rewritten");
            encoded_password = password.replace("%2A", "*");
        } else {
            encoded_password = urlencoding::encode(&password).into_owned();
        }

        // Send an HTTP GET request to get the CSRF token
        let resp = self
            .reqwest_client
            .get("https://www.reddit.com/login/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.5615.121 Safari/537.36")
            .send()
            .await
            .expect("Failed to send HTTP request; to obtain CSRF token");

        debug!("CSRF Request Response: {:?}", resp);
        let body = resp.text();
        let body = body.await.expect("Failed to read response body");

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
        let _x = self.reqwest_client
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
        .await
        .expect("Failed to send HTTP request; to obtain session token");

        // Request / to get the bearer token
        let response = self.reqwest_client
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
        .await
        .expect("Error getting bearer token");

        // Extract the Bearer Token from the JSON response
        let bearer_regex = Regex::new(r#"accessToken":"([^"]+)"#).unwrap();

        let mut bearer_token: String = String::default();
        for i in bearer_regex.captures_iter(&response.text().await.unwrap()) {
            for i in i.get(1).iter() {
                bearer_token = String::from(i.as_str().clone());
            }
        }

        // Login to matrix.reddit.com using the bearer for reddit.com
        let data = format!(
        "{{\"type\":\"com.reddit.token\",\"token\":\"{bearer_token}\",\"initial_device_display_name\":\"Reddit Web Client\"}}"
        );

        debug!("Matrix request body: {:?}", data);

        let response = self.reqwest_client
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
        .await
        .expect("Failed to send HTTP request; to login to matrix");

        debug!("Matrix login response: {:#?}", response);
        if !response.status().is_success() {
            println!("{}", style("Login failed").red().bold());
            crate::exit!(0, "Login exited with failure");
        }

        self.bearer = Some(bearer_token);
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "creds"]
    async fn login() {
        let mut client = super::super::new_client(true);
        let (username, password) = get_login();

        client.login(username, password).await;
    }

    fn get_login() -> (String, String) {
        let username = std::env::var("REXIT_USERNAME").expect("Could not find username in env");
        let password = std::env::var("REXIT_PASSWORD").expect("Could not find password in env");

        (username, password)
    }
}
