use ::failure::{format_err, Error};
use ::serde_derive::{Deserialize, Serialize};
use ::uuid::Uuid;
use ::serde_json;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Add,
    Remove,
    Switch,

    Start,
    Pause,
    Kill,

    Status,
    Reset,
    Clear,

    Invalid,
}

/// The representation of a message
#[derive(Serialize, Deserialize)]
pub struct Message {
    pub socket_uuid: Uuid,
    pub message_type: MessageType,
    pub payload: String,
    pub add: Option<AddMessage>,
}

impl Default for Message {
    fn default() -> Message {
        Message {
            socket_uuid: Uuid::new_v4(),
            message_type: MessageType::Invalid,
            payload: String::from(""),
            add: None,
            remove: None,
            switch: None,
        }
    }
}

/// Get the message index depending on the message type
/// This is needed to signal a receiver which kind of message he should expect.
/// This u64 is sent in the header
pub fn get_message_index(message_type: &MessageType) -> u64 {
    match message_type {
        MessageType::Add => 1,
        MessageType::Remove => 2,
        MessageType::Switch => 3,

        MessageType::Start => 10,
        MessageType::Pause => 11,
        MessageType::Kill => 12,

        MessageType::Status => 20,
        MessageType::Reset => 21,
        MessageType::Clear => 22,

        MessageType::Invalid => panic!("Found invalid MessageType"),
    }
}

/// The counterpart to get_message_index
/// Resolve a given message index to the correct message type
pub fn get_message_type(message_index: usize) -> Result<MessageType, Error> {
    match message_index {
        1 => Ok(MessageType::Add),
        2 => Ok(MessageType::Remove),
        3 => Ok(MessageType::Switch),

        10 => Ok(MessageType::Start),
        11 => Ok(MessageType::Pause),
        12 => Ok(MessageType::Kill),

        20 => Ok(MessageType::Status),
        21 => Ok(MessageType::Reset),
        22 => Ok(MessageType::Clear),

        _ => Err(format_err!("Found invalid message index for MessageType")),
    }
}

/// Create a `Message` depending on the message_type
pub fn extract_message(message_type: MessageType, message: String) -> Message {
    match message_type {
        // Handle the Add message
        MessageType::Add => {
            let result = serde_json::from_str::<AddMessage>(&message);

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
