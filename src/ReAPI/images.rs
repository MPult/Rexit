use console::style;
use std::path::PathBuf;
use crate::exit;

pub struct Image {
    pub extension: String,
    pub id: String,
    data: Vec<u8>,
}

impl Image {
    pub fn export_to(&self, path: PathBuf) {
        let mut path = path;
        path.push(self.id);

        std::fs::write(path.with_extension(self.extension), self.data).unwrap();
    }
}

/// Gets images from a mxc:// URL as per [SPEC](https://spec.matrix.org/v1.6/client-server-api/#get_matrixmediav3downloadservernamemediaid)
pub fn get_image(url: String, debug: bool) -> Image {
    let client = super::new_debug_client(debug);
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
                    crate::exit!(0);
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

    let data = data.bytes().unwrap().to_vec();

    Image {
        extension: extension.unwrap(),
        id,
        data
    }
}

fn parse_matrix_image_url(url: &str) -> (String, String) {
    let url = reqwest::Url::parse(url).unwrap(); // I assume that all urls given to this function are valid

    let output_url =
        reqwest::Url::parse("https://matrix.redditspace.com/_matrix/media/r0/download/reddit.com/").unwrap();

    let id = url.path_segments().unwrap().next().unwrap();

    let output_url = output_url.join(id).unwrap();

    (output_url.to_string(), id.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn get_image() {
        super::get_image("mxc://dwdprq7pxbva1".to_string(), true);
    }
}