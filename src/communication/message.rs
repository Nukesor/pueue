pub enum MessageType {
    Add,
}

pub struct Message {
    pub message_type: MessageType,
    pub payload: String,
    pub add: Option<AddMessage>,
}

pub fn get_command_index(message_type: &MessageType) -> u64 {
    match message_type {
        MessageType::Add => 1,
    }
}

pub fn get_command_type(message_index: u64) -> MessageType {
    match message_index {
         1 => MessageType::Add,
        _ => panic!("Found invalid"),
    }
}

#[derive(Serialize, Deserialize)]
pub struct AddMessage {
    pub command: String,
    pub path: String,
}
