#[derive(Clone, Debug)]
pub enum MessageType {
    Add,
}

/// The representation of a message
/// It consists of
pub struct Message {
    pub message_type: MessageType,
    pub payload: String,
    pub add: Option<AddMessage>,
}

/// Get the message index depending on the message type
/// This is needed to signal a receiver which kind of message he should expect.
/// This u64 is sent in the header
pub fn get_message_index(message_type: &MessageType) -> u64 {
    match message_type {
        MessageType::Add => 1,
    }
}

/// The counterpart to get_message_index
/// Resolve a given message index to the correct message type
pub fn get_message_type(message_index: usize) -> Result<MessageType, String> {
    match message_index {
        1 => Ok(MessageType::Add),
        _ => Err("Found invalid message index for MessageType".to_string()),
    }
}

/// The Message used to add a new command to the daemon.
#[derive(Serialize, Deserialize)]
pub struct AddMessage {
    pub command: String,
    pub path: String,
}
