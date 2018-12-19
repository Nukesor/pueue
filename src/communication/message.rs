use ::serde_derive::{Deserialize, Serialize};

/// The Message used to add a new command to the daemon.
#[derive(Serialize, Deserialize)]
pub enum Message {
    Add(AddMessage),
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

#[derive(Serialize, Deserialize)]
pub struct AddMessage {
    pub command: String,
    pub path: String,
}
