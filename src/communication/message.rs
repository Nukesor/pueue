pub enum MessageTypes {
    Add,
}

pub struct Message {
    pub message_type: MessageTypes,
    pub payload: String,
    pub add: Option<AddMessage>,
}

#[derive(Serialize, Deserialize)]
pub struct AddMessage {
    pub command: String,
    pub path: String,
}
