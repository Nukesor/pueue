use ::serde_derive::{Deserialize, Serialize};

use crate::state::State;

/// The Message used to add a new command to the daemon.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Add(AddMessage),
    Remove(RemoveMessage),
    Switch(SwitchMessage),
    Stash(StashMessage),
    Enqueue(EnqueueMessage),

    Start(StartMessage),
    Restart(RestartMessage),
    Pause(PauseMessage),
    Kill(KillMessage),

    Reset,
    Clean,

    Status,
    StatusResponse(State),
    Success(String),
    Failure(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddMessage {
    pub command: String,
    pub arguments: Vec<String>,
    pub path: String,
    pub start_immediately: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoveMessage {
    pub task_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StashMessage {
    pub task_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnqueueMessage {
    pub task_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwitchMessage {
    pub command: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StartMessage {
    pub task_ids: Option<Vec<i32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RestartMessage {
    pub task_ids: Vec<i32>,
    pub start_immediately: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PauseMessage {
    pub wait: bool,
    pub task_ids: Option<Vec<i32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KillMessage {
    pub all: bool,
    pub task_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextMessage {
    pub text: String,
}

pub fn create_success_message(text: String) -> Message {
    Message::Success(text)
}

pub fn create_failure_message(text: String) -> Message {
    Message::Failure(text)
}
