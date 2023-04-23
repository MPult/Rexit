use super::exit;
use console::style;
use reqwest::blocking::Client;
use std::path::PathBuf;
use url::Url;

pub fn export_image(client: &Client, url: String) {
    info!(target: "export_image", "Getting image: {}", url);
    let (url, id) = parse_matrix_image_url(url.as_str());

    let data = client.get(url).send().unwrap();

    let mut extension: Option<String> = None;
    for (header_name, header_value) in data.headers() {
        if header_name.as_str() == "content-type" {
            let file_type = header_value.to_str().unwrap().to_string();

            let mut file_type = file_type.split("/");

            extension = match file_type.nth(1).unwrap() {
                "jpeg" => Some("jpeg".to_string()),
                "png" => Some("png".to_string()),
                "gif" => Some("gif".to_string()),
                _ => {
                    exit!(0);
                }
            };
        }
    }
    if extension.is_none() {
        println!(
            "{}",
            style("Error: Something failed reading the image type").red()
        );
        error!("Something failed reading the image type");
        exit!(0);
    }

    let data = data.bytes().unwrap();

    let mut output_path = PathBuf::from("./images/");
    output_path.push(id);

    std::fs::write(output_path.with_extension(extension.unwrap()), data).unwrap();
}

fn parse_matrix_image_url(url: &str) -> (String, String) {
    let url = Url::parse(url).unwrap(); // I assume that all urls given to this function are valid

    let output_url =
        Url::parse("https://matrix.redditspace.com/_matrix/media/r0/download/reddit.com/").unwrap();

    let id = url.path_segments().unwrap().next().unwrap();

    let output_url = output_url.join(id).unwrap();

    (output_url.to_string(), id.to_string())
}
