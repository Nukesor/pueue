pub enum MessageType {
    Add,
}

pub struct Message {
    pub message_type: MessageType,
    pub payload: String,
    pub add: Option<AddMessage>,
}

pub fn get_message_index(message_type: &MessageType) -> u64 {
    match message_type {
        MessageType::Add => 1,
    }
}

pub fn get_message_type(message_index: usize) -> Result<MessageType, String> {
    match message_index {
        1 => Ok(MessageType::Add),
//        _ => Err(String:from("Found invalid message index for MessageType"));
    }
}

#[derive(Serialize, Deserialize)]
pub struct AddMessage {
    pub command: String,
    pub path: String,
}
