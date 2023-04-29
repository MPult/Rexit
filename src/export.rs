use crate::ReAPI;

pub fn export_room_chats_txt(room: ReAPI::Room) {
    let mut output_buffer: String = String::new();
    let path = format!("./out/{}.txt", room.id);

    for message in room.messages() {
        if let ReAPI::Content::Message(text) = message.content {
            let line: String = format!(
                "[{}] {}: {}\n",
                message.timestamp.to_rfc3339().to_string(), message.author, text
            );
        
            output_buffer.push_str(line.as_str());
        } 
        else if let ReAPI::Content::Image(image) = message.content {
            let line: String = format!(
                "[{}] {}: {}\n",
                message.timestamp.to_rfc3339_opts(chrono::SecondsFormat::Secs, true).to_string(), message.author, image.id
            );

            output_buffer.push_str(line.as_str()); 

            std::fs::write(format!("./out/images/{}{}", image.id, image.extension), image.data).unwrap();
        }
    }

    std::fs::write(path, output_buffer).unwrap();
}

