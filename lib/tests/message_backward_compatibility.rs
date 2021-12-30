use serde_cbor::de::from_slice;
use serde_cbor::ser::to_vec;
use serde_derive::{Deserialize, Serialize};

use pueue_lib::network::message::Message as OriginalMessage;

/// This is the main message enum. \
/// Everything that's communicated in Pueue can be serialized as this enum.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    Switch(SwitchMessage),
    Clean(CleanMessage),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SwitchMessage {
    pub task_id_1: usize,
    pub task_id_2: usize,
    pub some_new_field: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CleanMessage {}

#[test]
/// Make sure we can deserialize old messages as long as we have default values set.
fn test_deserialize_old_message() {
    let message = Message::Clean(CleanMessage {});
    let payload_bytes = to_vec(&message).unwrap();

    let message: OriginalMessage = from_slice(&payload_bytes).unwrap();
    if let OriginalMessage::Clean(message) = message {
        // The serialized message didn't have the `successful_only` property yet.
        // Instead the default `false` should be used.
        assert!(!message.successful_only);
    } else {
        panic!("It must be a clean message");
    }
}

#[test]
/// Make sure we can deserialize new messages, even if new values exist.
fn test_deserialize_new_message() {
    let message = Message::Switch(SwitchMessage {
        task_id_1: 0,
        task_id_2: 1,
        some_new_field: 2,
    });
    let payload_bytes = to_vec(&message).unwrap();

    let message: OriginalMessage = from_slice(&payload_bytes).unwrap();
    // The serialized message did have an additional field. The deserialization works anyway.
    assert!(matches!(message, OriginalMessage::Switch(_)));
}
