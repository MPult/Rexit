use std::fmt::format;
use std::fs::OpenOptions;
use std::io::Write;
use std::{fs, io};

use crate::AllChats;

// Function to check what export format is desired and calls the apporopriate function below
pub fn decide_export(all_chats: AllChats) {
    // For now just defualt to exporting txt
    export_to_txt(all_chats.clone());
    export_to_json(all_chats.clone());
    export_to_csv(all_chats);
}

// Function to export the chats into .txt files
pub fn export_to_txt(all_chats: AllChats) {

    // Iterate over the individual chats / rooms (idk what to call it reddit uses the terms interchangibly)
    for chat in all_chats.chats {
        // Create the file for each chat / room
        let filename = std::path::PathBuf::from(&chat.ID[1..10]).with_extension("txt");
        std::fs::write(filename.clone(), "").unwrap();

        // Iterate over each message in the chat; append to the file
        for message in chat.messages {
            // Format for the line to be apended
            let line: String = format!(
                "[{}] {}: {}",
                message.timestamp, message.author, message.message
            );

            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(filename.clone())
                .unwrap();

            if let Err(e) = writeln!(file, "{}", line) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
    }
}

// Function to export the chats into .json files
pub fn export_to_json(all_chats: AllChats) {
    let file_data = serde_json::to_string(&all_chats).unwrap();

    fs::write("export.json", file_data).expect("Unable to write file");
}

// Function to export the chats into .csv files
pub fn export_to_csv(all_chats: AllChats) {

    // Iterate over the individual chats / rooms (idk what to call it reddit uses the terms interchangibly)
    for chat in all_chats.chats {
        // Create the file for each chat / room
        let filename = std::path::PathBuf::from(&chat.ID[1..10]).with_extension("csv");
        std::fs::write(filename.clone(), "timestamp, author, message \n").unwrap();

        // Iterate over each message in the chat; append to the file
        for message in chat.messages {
            // Format for the line to be apended
            let line: String = format!(
                "{}, {}, {}",
                message.timestamp, message.author, message.message
            );

            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(filename.clone())
                .unwrap();

            if let Err(e) = writeln!(file, "{}", line) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
    }
}