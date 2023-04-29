use super::Client;
use crate::exit;
use cached::SizedCache;
use console::style;
use serde::Serialize;
use std::path::PathBuf;

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
#[cached::proc_macro::cached(
    type = "SizedCache<String, Image>",
    create = "{ SizedCache::with_size(10_000) }",
    convert = r#"{ format!("{}", url) }"#
)]
pub fn get_image(client: &Client, url: String) -> Image {
    info!(target: "get_image", "Getting image: {}", url);
    let (url, id) = parse_matrix_image_url(url.as_str());

    let data = client.reqwest_client.get(url).send().unwrap();

    Image {
        extension: get_image_extension(&data.headers()),
        id,
        data: data.bytes().unwrap().to_vec(),
    }
}

fn parse_matrix_image_url(url: &str) -> (String, String) {
    let url = reqwest::Url::parse(url).unwrap(); // I assume that all urls given to this function are valid

    println!("{}", url);
    let output_url =
        reqwest::Url::parse("https://matrix.redditspace.com/_matrix/media/r0/download/reddit.com/")
            .unwrap();

    let id = url.path_segments().unwrap().next().unwrap();

    let output_url = output_url.join(id).unwrap();

    (output_url.to_string(), id.to_string())
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

#[cfg(test)]
mod tests {
    #[test]
    fn get_image() {
        let image = super::get_image(
            &super::super::new_client(true),
            "mxc://reddit.com/dwdprq7pxbva1/".to_string(),
        );

        image.export_to(std::path::PathBuf::from(
            "./test_resources/test_cases/ReAPI/images/get_images/",
        ));

        assert!(std::path::PathBuf::from(
            "./test_resources/test_cases/ReAPI/images/get_images/dwdprq7pxbva1.gif"
        )
        .exists());

        std::fs::remove_file(
            "./test_resources/test_cases/ReAPI/images/get_images/dwdprq7pxbva1.gif",
        )
        .expect("Could not remove downloaded file");
    }
}
