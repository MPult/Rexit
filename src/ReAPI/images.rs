use super::Client;
use crate::{exit, image_log};
use console::style;
use log::{error, info};
use serde::Serialize;
use std::path::PathBuf;
use url::Url;

#[derive(std::hash::Hash, Clone, Debug, Serialize)]
pub struct Image {
    pub extension: String,
    pub id: String,
    pub data: Vec<u8>,
}

impl Image {
    pub fn export_to(&self, path: PathBuf) {
        let mut path = path;
        path.push(self.id.clone());

        std::fs::write(
            path.with_extension(self.extension.clone()),
            self.data.clone(),
        )
        .unwrap();
    }

    pub fn from(id: String, extension: String, data: Vec<u8>) -> Image {
        Image {
            extension,
            id,
            data,
        }
    }
}

/// Gets images from a mxc:// URL as per [SPEC](https://spec.matrix.org/v1.6/client-server-api/#get_matrixmediav3downloadservernamemediaid)
pub async fn get_image(client: &Client, url: String, out: PathBuf, path: &std::path::Path, redact: bool) {
    let mut url = url;
    let mut id: Option<String> = None;
    
    
    if image_log::check_image_log(out.clone(), url.clone()) {
        // Image was already downloaded
        info!("Image was already downloaded; Skipping");
        return;
    }
    
    // Handle redaction
    if redact  {
      info!(target: "get_image", "Getting image: [REDACTED]");
    } else {
      info!(target: "get_image", "Getting image: {}...", &url[0..30]);
    }
    image_log::write_image_log(out, url.clone());


    if url.starts_with("mxc") {
        // Matrix images
        (url, id) = parse_matrix_image_url(url.as_str());
        let data = client.reqwest_client.get(url.clone()).send().await.unwrap();
        let path = path
            .join(id.unwrap())
            .with_extension(get_image_extension(data.headers()));

        std::fs::write(path, data.bytes().await.unwrap().to_vec()).unwrap();
    } else {
        // Litteraly any other image
        // Parse the image url to get the ID
        id = Some(Url::parse(&url).unwrap().path().to_string());
        id = Some(id.unwrap().replace("/", ""));

        let data = client.reqwest_client.get(url.clone()).send().await.unwrap();
        let path = path.join(id.unwrap());

        std::fs::write(path, data.bytes().await.unwrap().to_vec()).unwrap();
    }
}

fn parse_matrix_image_url(url: &str) -> (String, Option<String>) {
    let url = reqwest::Url::parse(url).unwrap(); // I assume that all urls given to this function are valid

    let output_url =
        reqwest::Url::parse("https://matrix.redditspace.com/_matrix/media/r0/download/reddit.com/")
            .unwrap();

    let id = url.path_segments().unwrap().next().unwrap();

    let output_url = output_url.join(id).unwrap();

    (output_url.to_string(), Some(id.to_string()))
}

fn get_image_extension(headers: &reqwest::header::HeaderMap) -> String {
    let mut extension: Option<String> = None;

    // Iterate over headers to find content-type
    for (header_name, header_value) in headers {
        if header_name.as_str() != "content-type" {
            continue;
        }
        let file_type = header_value.to_str().unwrap().to_string();

        let mut file_type = file_type.split("/");

        extension = match file_type.nth(1).unwrap() {
            "jpeg" => Some("jpeg".to_string()),
            "png" => Some("png".to_string()),
            "gif" => Some("gif".to_string()),
            _ => {
                println!("{}", style("Failed to read image type").red().bold());
                exit!(0);
            }
        };
    }

    if extension.is_none() {
        println!(
            "{}",
            style("Error: Something failed reading the image type")
                .red()
                .bold()
        );
        error!("Something failed reading the image type");
        exit!(0);
    }

    return extension.unwrap();
}