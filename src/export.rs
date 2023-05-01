use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::ReAPI::{self, Post};

/// Export the chats into a .txt file
pub fn export_room_chats_txt(room: ReAPI::Room) {
    let mut output_buffer: String = String::new();
    let path = format!("./out/messages/{}.txt", &room.id[1..10]);

    for message in room.messages() {
        if let ReAPI::Content::Message(text) = message.content {
            let line: String = format!(
                "[{}] {}: {}\n",
                message
                    .timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                    .to_string(),
                message.author,
                text
            );

            output_buffer.push_str(line.as_str());
        } else if let ReAPI::Content::Image(image) = message.content {
            let image_text = format!("FILE: {}", image.id);

            let line: String = format!(
                "[{}] {}: {}\n",
                message
                    .timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                    .to_string(),
                message.author,
                image_text
            );

            output_buffer.push_str(line.as_str());
        }
    }

    std::fs::write(path, output_buffer).unwrap();
}

/// Export the chats into .json files.
pub fn export_room_chats_json(room: ReAPI::Room) {
    let path = format!("./out/messages/{}.json", &room.id[1..10]);

    let file_data = serde_json::to_string(&room).unwrap();

    fs::write(path, file_data).expect("Unable to write file");
}

/// Export chats into csv
pub fn export_room_chats_csv(room: ReAPI::Room) {
    // Create the file for each chat / room
    let path = format!("./out/messages/{}.csv", &room.id[1..10]);

    std::fs::write(path.clone(), "timestamp, author, message \n").unwrap();

    // Iterate over each message in the chat; append to the file
    for message in room.messages() {
        // Format for the line to be appended
        let mut line: String = String::new();

        if let ReAPI::Content::Message(text) = message.content {
            line = format!(
                "{}, {}, {},",
                message
                    .timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                    .to_string(),
                message.author,
                text
            );
        } else if let ReAPI::Content::Image(image) = message.content {
            let image_text = format!("FILE: {}", image.id);

            line = format!(
                "{}, {}, {},",
                message
                    .timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                    .to_string(),
                message.author,
                image_text
            );
        }

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(path.clone())
            .unwrap();

        if let Err(e) = writeln!(file, "{}", line) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
}

/// Export images from chats
pub fn export_room_images(room: ReAPI::Room) {
    for message in room.messages() {
        if let ReAPI::Content::Image(image) = message.content {
            std::fs::write(
                format!("./out/messages/images/{}.{}", image.id, image.extension),
                image.data,
            )
            .unwrap();
        }
    }
}

/// Export saved posts
pub fn export_saved_posts(post_array: Vec<Post>, formats: Vec<&str>) {
    // Export to JSON
    if formats.contains(&"json") {
        let path = "./out/saved_posts/saved_posts.json";

        let file_data = serde_json::to_string(&post_array).unwrap();

        fs::write(path, file_data).expect("Unable to write file");
    }

    // Export to txt
    if formats.contains(&"txt") {
        let path = "./out/saved_posts/saved_posts.txt";
        let mut output_buffer: String = String::new();

        for post in &post_array {
            // Iterate over each line and append to .txt file
            let line: String = format!(
                "Title: {}, Subreddit: {}, Permalink: {}, Images {:?}\n",
                post.title, post.subreddit_name, post.permalink, post.img_url
            );

            output_buffer.push_str(line.as_str());
        }
        std::fs::write(path, output_buffer).unwrap();
    }

    if formats.contains(&"csv") {
        // Export to CSV
        let path = "./out/saved_posts/saved_posts.csv";
        let mut output_buffer: String = "Title, Subreddit, Permalink, Images\n".to_owned();

        for post in post_array {
            // Iterate over each line and append to .txt file
            let line: String = format!(
                "{}, {}, {}, {:?}\n",
                post.title, post.subreddit_name, post.permalink, post.img_url
            );

            output_buffer.push_str(line.as_str());
        }
        std::fs::write(path, output_buffer).unwrap();
    }
}
