use ::chrono::prelude::*;
use ::serde_derive::{Deserialize, Serialize};
use ::std::collections::BTreeMap;

use crate::state::State;
use crate::task::Task;

/// The Message used to add a new command to the daemon.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Add(AddMessage),
    Remove(Vec<usize>),
    Switch(SwitchMessage),
    Stash(Vec<usize>),
    Enqueue(EnqueueMessage),

    Start(Vec<usize>),
    Restart(RestartMessage),
    Pause(PauseMessage),
    Kill(KillMessage),

    Send(SendMessage),
    EditRequest(usize),
    EditResponse(EditResponseMessage),
    Edit(EditMessage),

    Status,
    StatusResponse(State),
    Log(Vec<usize>),
    LogResponse(BTreeMap<usize, TaskLogMessage>),
    Stream(String),
    StreamRequest(StreamRequestMessage),
    Reset,
    Clean,
    DaemonShutdown,

    Success(String),
    Failure(String),

    Parallel(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddMessage {
    pub command: String,
    pub path: String,
    pub start_immediately: bool,
    pub stashed: bool,
    pub enqueue_at: Option<DateTime<Local>>,
    pub dependencies: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwitchMessage {
    pub task_id_1: usize,
    pub task_id_2: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnqueueMessage {
    pub task_ids: Vec<usize>,
    pub enqueue_at: Option<DateTime<Local>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RestartMessage {
    pub task_ids: Vec<usize>,
    pub start_immediately: bool,
    pub stashed: bool,
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
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditResponseMessage {
    pub task_id: usize,
    pub command: String,
    pub path: String,
}

// The booleans decides, whether the stream should be continuous or a oneshot
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamRequestMessage {
    pub task_id: usize,
    pub follow: bool,
    pub err: bool,
}

/// Helper struct for sending tasks and their log output to the client
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskLogMessage {
    pub task: Task,
    pub stdout: String,
    pub stderr: String,
}

pub fn create_success_message<T: ToString>(text: T) -> Message {
    Message::Success(text.to_string())
}

pub fn create_failure_message<T: ToString>(text: T) -> Message {
    Message::Failure(text.to_string())
}
