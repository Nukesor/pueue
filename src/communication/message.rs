use serde_json;

#[derive(Clone, Debug)]
pub enum MessageType {
    Add,
    Invalid,
}

/// The representation of a message
/// It consists of
pub struct Message {
    pub message_type: MessageType,
    pub payload: String,
    pub add: Option<AddMessage>,
}

impl Default for Message {
    fn default() -> Message {
        Message {
            message_type: MessageType::Invalid,
            payload: String::from(""),
            add: None,
        }
    }
}

/// Get the message index depending on the message type
/// This is needed to signal a receiver which kind of message he should expect.
/// This u64 is sent in the header
pub fn get_message_index(message_type: &MessageType) -> u64 {
    match message_type {
        MessageType::Add => 1,
        MessageType::Invalid => panic!("Found invalid MessageType"),
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

/// Create a `Message` depending on the message_type
pub fn extract_message(message_type: &MessageType, message: String) -> Message {
    match message_type {
        // Handle the Add message
        MessageType::Add => {
            let result = serde_json::from_str(&message);

            let add_message = if let Ok(add_message) = result {
                add_message
            } else {
                println!("{:?}", result.err());
                panic!("Found invalid ");
            };
            Message {
                message_type: MessageType::Add,
                payload: message,
                add: Some(add_message),
                ..Default::default()
            }
        }
        MessageType::Invalid => panic!("Invalid message type"),
    }
}

/// The Message used to add a new command to the daemon.
#[derive(Serialize, Deserialize)]
pub struct AddMessage {
    pub command: String,
    pub path: String,
}
