use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::ReAPI::{self, Message};

/// Export the chats into a .txt file
pub fn export_room_chats_txt(room: ReAPI::Room) {
    let mut output_buffer: String = String::new();
    let path = format!("./out/{}.txt", &room.id[1..10]);

    for message in room.messages() {
        if let ReAPI::Content::Message(text) = message.content {
            let line: String = format!(
                "[{}] {}: {}\n",
                message.timestamp.to_rfc3339_opts(chrono::SecondsFormat::Secs, true).to_string(),
                message.author,
                text
            );

            output_buffer.push_str(line.as_str());
        } else if let ReAPI::Content::Image(image) = message.content {
            let line: String = format!(
                "[{}] {}: {}\n",
                message
                    .timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                    .to_string(),
                message.author,
                image.id
            );

            output_buffer.push_str(line.as_str());

            std::fs::write(
                format!("./out/images/{}{}", image.id, image.extension),
                image.data,
            )
            .unwrap();
        }
    }

    std::fs::write(path, output_buffer).unwrap();
}

/// Export the chats into .json files.
pub fn export_room_chats_json(room: ReAPI::Room) {
    let path = format!("./out/{}.json", &room.id[1..10]);

    let file_data = serde_json::to_string(&room).unwrap();

    fs::write(path, file_data).expect("Unable to write file");
}

pub fn export_room_chats_csv(room: ReAPI::Room) {
    // Create the file for each chat / room
    let path = format!("./out/{}.csv", &room.id[1..10]);

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
            line = format!(
                "{}, {}, {},",
                message
                    .timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                    .to_string(),
                message.author,
                image.id
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
