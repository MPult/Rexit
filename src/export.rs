use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::ReAPI;

/// Export the chats into a .txt file
pub fn export_room_chats_txt(room: ReAPI::Room, out_folder: &Path) {
    let mut output_buffer: String = String::new();
    let path = out_folder.join(format!("messages/{}.txt", room.file_stem()));

    for message in room.messages() {
        let line: String = format!(
            "[{}] {}: {}\n",
            message
                .timestamp
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            message.author,
            message.content.as_text()
        );

        output_buffer.push_str(line.as_str());
    }

    std::fs::write(path, output_buffer).unwrap();
}

/// Export the chats into .json files.
pub fn export_room_chats_json(room: ReAPI::Room, out_folder: &Path) {
    let path = out_folder.join(format!("messages/{}.json", room.file_stem()));

    let file_data = serde_json::to_string(&room).unwrap();

    fs::write(path, file_data).expect("Unable to write file");
}

/// Export chats into csv
pub fn export_room_chats_csv(room: ReAPI::Room, out_folder: &Path) {
    // Create the file for each chat / room
    let path = out_folder.join(format!("messages/{}.csv", room.file_stem()));

    std::fs::write(path.clone(), "timestamp, author, message \n").unwrap();

    // Iterate over each message in the chat; append to the file
    for message in room.messages() {
        // Format for the line to be appended
        let line = format!(
            "{}, {}, {},",
            message
                .timestamp
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            message.author,
            message.content.as_text()
        );

        let mut file = OpenOptions::new().append(true).open(path.clone()).unwrap();

        if let Err(e) = writeln!(file, "{}", line) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
}

/// Metadata written next to the chat exports, read by `rexit view`
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Metadata {
    pub schema_version: u32,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    /// Matrix user id of the account that was exported
    pub user_id: Option<String>,
    pub rooms: Vec<MetadataRoom>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MetadataRoom {
    pub id: String,
    /// File name (without extension) of the room's export
    pub file: String,
}

/// Current version of the chat export format
pub const SCHEMA_VERSION: u32 = 2;

/// Export the metadata.json for a chat export
pub fn export_metadata(rooms: &[ReAPI::Room], user_id: Option<String>, out_folder: &Path) {
    let metadata = Metadata {
        schema_version: SCHEMA_VERSION,
        exported_at: chrono::Utc::now(),
        user_id,
        rooms: rooms
            .iter()
            .map(|room| MetadataRoom {
                id: room.id.clone(),
                file: room.file_stem(),
            })
            .collect(),
    };
    let path = out_folder.join("messages/metadata.json");
    fs::write(path, serde_json::to_string_pretty(&metadata).unwrap())
        .expect("Unable to write file");
}

/// Export saved posts
pub fn export_saved_posts(
    post_array: Vec<ReAPI::saved_posts::SavedPost>,
    formats: Vec<&str>,
    out_folder: &Path,
) {
    // Export to JSON
    if formats.contains(&"json") {
        let path = out_folder.join("saved_posts/saved_posts.json");

        let file_data = serde_json::to_string(&post_array).unwrap();

        fs::write(path, file_data).expect("Unable to write file");
    }

    // Export to txt
    if formats.contains(&"txt") {
        let path = out_folder.join("saved_posts/saved_posts.txt");
        let mut output_buffer: String = String::new();

        for post in &post_array {
            // Iterate over each line and append to .txt file
            let line: String = format!(
                "Title: {}, Subreddit: {}, Body: {} Permalink: {}, Images {:?}\n",
                post.title, post.subreddit_name, post.body_text, post.permalink, post.img_url
            );

            output_buffer.push_str(line.as_str());
        }
        std::fs::write(path, output_buffer).unwrap();
    }

    if formats.contains(&"csv") {
        // Export to CSV
        let path = out_folder.join("saved_posts/saved_posts.csv");
        let mut output_buffer: String = "Title, Subreddit, Body, Permalink, Images\n".to_owned();

        for post in post_array {
            // Iterate over each line and append to .txt file
            let line: String = format!(
                "{}, {}, {}, {}, {:?}\n",
                post.title, post.subreddit_name, post.body_text, post.permalink, post.img_url
            );

            output_buffer.push_str(line.as_str());
        }
        std::fs::write(path, output_buffer).unwrap();
    }
}

/// Export subreddit
pub fn export_subreddit(
    post_array: Vec<ReAPI::subreddit::Post>,
    formats: Vec<&str>,
    out_folder: &Path,
) {
    // Export to JSON
    if formats.contains(&"json") {
        let path = out_folder.join("subreddit/subreddit.json");

        let file_data = serde_json::to_string(&post_array).unwrap();

        fs::write(path, file_data).expect("Unable to write file");
    }

    // Export to txt
    if formats.contains(&"txt") {
        let path = out_folder.join("subreddit/subreddit.txt");
        let mut output_buffer: String = String::new();

        for post in &post_array {
            // Iterate over each line and append to .txt file
            let line: String = format!(
                "Title: {}, Subreddit: {}, Body: {}, Permalink: {}, Images {:?}\n",
                post.title, post.subreddit_name, post.body_text, post.permalink, post.img_url
            );

            output_buffer.push_str(line.as_str());
        }
        std::fs::write(path, output_buffer).unwrap();
    }

    if formats.contains(&"csv") {
        // Export to CSV
        let path = out_folder.join("subreddit/subreddit.csv");
        let mut output_buffer: String = "Title, Subreddit, Permalink, Images\n".to_owned();

        for post in post_array {
            // Iterate over each line and append to .txt file
            let line: String = format!(
                "{}, {}, {}, {}, {:?}\n",
                post.title, post.subreddit_name, post.body_text, post.permalink, post.img_url
            );

            output_buffer.push_str(line.as_str());
        }
        std::fs::write(path, output_buffer).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use chrono::Utc;

    use crate::ReAPI;

    // return a path from an env var with suffix or use tempdir
    fn out_dir(suffix: &str) -> Box<dyn AsRef<Path>> {
        match std::env::var("REXIT_TEST_OUT_DIR") {
            Ok(dir) => Box::new(PathBuf::from(dir).join(suffix)),
            Err(_) => Box::new(tempfile::tempdir().unwrap()),
        }
    }

    #[test]
    fn export_room_chats() {
        let out_dir = out_dir("export_room_chats");
        let out_path = out_dir.as_ref().as_ref();

        std::fs::create_dir_all(out_path.join("messages/images")).unwrap();

        let messages_array: Option<Vec<ReAPI::Message>> = Some(Vec::new());

        let message = ReAPI::Message {
            id: "$1".to_owned(),
            author: "rexitTest".to_owned(),
            sender: "@t2_test:reddit.com".to_owned(),
            timestamp: Utc::now(),
            content: ReAPI::Content::Message("Testing".to_owned()),
        };
        messages_array.clone().unwrap().push(message);

        let room = ReAPI::Room {
            id: "!fTxOL9GzJaZR71aLRSYstHNVR5j_Zi82L4hIVyjdHuw:reddit.com".to_owned(),
            name: None,
            members: vec![],
            messages: messages_array,
        };

        // Export it
        super::export_room_chats_csv(room.to_owned(), out_path);
        super::export_room_chats_txt(room.to_owned(), out_path);
        super::export_room_chats_json(room.to_owned(), out_path);
    }

    #[test]
    fn export_saved_posts() {
        let out_dir = out_dir("export_saved_posts");
        let out_path = out_dir.as_ref().as_ref();

        if out_path.exists() {
            std::fs::remove_dir_all(out_path).unwrap();
        }

        std::fs::create_dir_all(out_path.join("saved_posts")).unwrap();

        let mut posts: Vec<ReAPI::saved_posts::SavedPost> = Vec::new();

        let post = ReAPI::saved_posts::SavedPost {
            body_text: "hihi".to_owned(),
            title: "Da fehlt doch was".to_owned(),
            subreddit_name: "r/hamburg".to_owned(),
            permalink: "/r/hamburg/comments/134bv4v/da_fehlt_doch_was/".to_owned(),
            img_url: ["https://preview.redd.it/…051acd31351105e323c5d7a6".to_owned()].to_vec(),
        };
        posts.push(post);

        super::export_saved_posts(posts, ["txt", "json", "csv"].to_vec(), out_path)
    }
}
