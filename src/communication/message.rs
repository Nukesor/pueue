use ::serde_derive::{Deserialize, Serialize};

use crate::daemon::state::State;

/// The Message used to add a new command to the daemon.
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Add(AddMessage),
    Remove(RemoveMessage),
    Switch(SwitchMessage),

    Start(StartMessage),
    Pause(PauseMessage),
    Kill(KillMessage),

    Reset,
    Clear,

    Status,
    StatusResponse(State),
    Success(String),
    Failure(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddMessage {
    pub command: String,
    pub arguments: Vec<String>,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoveMessage {
    pub indices: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SwitchMessage {
    pub command: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartMessage {
    pub command: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PauseMessage {
    pub command: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KillMessage {
    pub command: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextMessage {
    pub text: String,
}

pub fn create_success_message(text: String) -> Message {
    Message::Success(text)
}

pub fn create_failure_message(text: String) -> Message {
    Message::Failure(text)
}
