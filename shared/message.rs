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

    Send(SendMessage),
    EditRequest(EditRequestMessage),
    EditResponse(EditResponseMessage),
    Edit(EditMessage),

    SimpleStatus,
    Status,
    Stream(String),
    // The boolean decides, whether the stream should be continuous or a oneshot
    StreamRequest(StreamRequestMessage),
    Reset,
    Clean,

    StatusResponse(State),
    Success(String),
    Failure(String),

    Parallel(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddMessage {
    pub command: String,
    pub path: String,
    pub start_immediately: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoveMessage {
    pub task_ids: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwitchMessage {
    pub task_id_1: usize,
    pub task_id_2: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StashMessage {
    pub task_ids: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnqueueMessage {
    pub task_ids: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StartMessage {
    pub task_ids: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RestartMessage {
    pub task_ids: Vec<usize>,
    pub start_immediately: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PauseMessage {
    pub wait: bool,
    pub task_ids: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KillMessage {
    pub all: bool,
    pub task_ids: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendMessage {
    pub task_id: usize,
    pub input: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditMessage {
    pub task_id: usize,
    pub command: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditRequestMessage {
    pub task_id: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditResponseMessage {
    pub task_id: usize,
    pub command: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamRequestMessage {
    pub task_id: usize,
    pub follow: bool,
    pub err: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextMessage {
    pub text: String,
}

pub fn create_success_message<T: ToString>(text: T) -> Message {
    Message::Success(text.to_string())
}

pub fn create_failure_message<T: ToString>(text: T) -> Message {
    Message::Failure(text.to_string())
}
