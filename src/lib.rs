//! This library is meant for testing only. Please do not use.

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

mod export;
mod cli;
mod login;
mod id_translation;
mod images;
mod messages;
mod macros;
mod ReAPI;

pub type RexitToken = String;
pub type Client = reqwest::blocking::Client;

pub use messages::AllChats;
pub use export::ExportFormat;

pub fn login(username: String, password: String) -> RexitToken {
    login::request_login(username, password, true)
}

pub fn get_all_messages(bearer: RexitToken, export_images: bool) -> AllChats {
    let rooms = messages::list_rooms(bearer.clone(), true);

    let all_chats = messages::iter_rooms(rooms, bearer, true, export_images);

    all_chats
}

pub fn export(format: ExportFormat, chats: AllChats) {
    match format {
        ExportFormat::CSV => export::export_to_csv(chats),
        ExportFormat::JSON => export::export_to_json(chats),
        ExportFormat::TXT => export::export_to_txt(chats)
    }
}