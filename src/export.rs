use crate::ReAPI;

pub fn export_room_chats(room: ReAPI::Room) {
    let output_buffer: String = String::new();

    for message in room.messages() {
        
        output_buffer.push_str()
    }
}