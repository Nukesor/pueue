use std::collections::{BTreeMap, HashMap};

use chrono::prelude::*;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use crate::state::{Group, State};
use crate::task::Task;

/// This is the main message enum. \
/// Everything that's communicated in Pueue can be serialized as this enum.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    Add(AddMessage),
    Remove(Vec<usize>),
    Switch(SwitchMessage),
    Stash(Vec<usize>),
    Enqueue(EnqueueMessage),

    Start(StartMessage),
    Restart(RestartMessage),
    Pause(PauseMessage),
    Kill(KillMessage),

    /// Used to send some input to a process's stdin
    Send(SendMessage),

    /// The first part of the three-step protocol to edit a task.
    /// This one requests an edit from the daemon.
    EditRequest(usize),
    /// This is send by the client if something went wrong during the editing process.
    /// The daemon will go ahead and restore the task's old state.
    EditRestore(usize),
    /// The daemon locked the task and responds with the task's details.
    EditResponse(EditResponseMessage),
    /// The client sends the edited details to the daemon.
    Edit(EditMessage),

    Group(GroupMessage),
    GroupResponse(GroupResponseMessage),

    Status,
    StatusResponse(Box<State>),
    Log(LogRequestMessage),
    LogResponse(BTreeMap<usize, TaskLogMessage>),

    /// The client requests a continuous stream of a task's log.
    StreamRequest(StreamRequestMessage),
    /// The next chunk of output, that's send to the client.
    Stream(String),

    /// The boolean decides, whether the children should be get a SIGTERM as well.
    Reset(ResetMessage),
    Clean(CleanMessage),
    DaemonShutdown(Shutdown),

    Success(String),
    Failure(String),

    Parallel(ParallelMessage),
}

/// This enum is used to express a selection of tasks.
/// As commands can be executed on various sets of tasks, we need some kind of datastructure to
/// explicitly and unambiguously specify the selection.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum TaskSelection {
    TaskIds(Vec<usize>),
    Group(String),
    All,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct AddMessage {
    pub command: String,
    pub path: String,
    pub envs: HashMap<String, String>,
    pub start_immediately: bool,
    pub stashed: bool,
    pub group: String,
    pub enqueue_at: Option<DateTime<Local>>,
    pub dependencies: Vec<usize>,
    pub label: Option<String>,
    pub print_task_id: bool,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct SwitchMessage {
    pub task_id_1: usize,
    pub task_id_2: usize,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct EnqueueMessage {
    pub task_ids: Vec<usize>,
    pub enqueue_at: Option<DateTime<Local>>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct StartMessage {
    pub tasks: TaskSelection,
    pub children: bool,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct RestartMessage {
    pub tasks: Vec<TasksToRestart>,
    pub start_immediately: bool,
    pub stashed: bool,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TasksToRestart {
    pub task_id: usize,
    pub command: String,
    pub path: String,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct PauseMessage {
    pub tasks: TaskSelection,
    pub wait: bool,
    pub children: bool,
}

/// This is a small custom Enum for all currently supported unix signals.
/// Supporting all unix signals would be a mess, since there is a LOT of them.
///
/// This is also needed for usage in clap, since nix's Signal doesn't implement [Display] and
/// [std::str::FromStr].
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize, Display, EnumString)]
pub enum Signal {
    #[strum(serialize = "SigInt", serialize = "sigint", serialize = "2")]
    SigInt,
    #[strum(serialize = "SigKill", serialize = "sigkill", serialize = "9")]
    SigKill,
    #[strum(serialize = "SigTerm", serialize = "sigterm", serialize = "15")]
    SigTerm,
    #[strum(serialize = "SigCont", serialize = "sigcont", serialize = "18")]
    SigCont,
    #[strum(serialize = "SigStop", serialize = "sigstop", serialize = "19")]
    SigStop,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct KillMessage {
    pub tasks: TaskSelection,
    pub children: bool,
    pub signal: Option<Signal>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct SendMessage {
    pub task_id: usize,
    pub input: String,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct EditMessage {
    pub task_id: usize,
    pub command: String,
    pub path: String,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct EditResponseMessage {
    pub task_id: usize,
    pub command: String,
    pub path: String,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum GroupMessage {
    Add {
        name: String,
        parallel_tasks: Option<usize>,
    },
    Remove(String),
    List,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct GroupResponseMessage {
    pub groups: BTreeMap<String, Group>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct ResetMessage {
    pub children: bool,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct CleanMessage {
    #[serde(default = "bool::default")]
    pub successful_only: bool,

    #[serde(default = "Option::default")]
    pub group: Option<String>,
}

/// Determines which type of shutdown we're dealing with.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum Shutdown {
    /// Emergency is most likely a system unix signal or a CTRL+C in a terminal.
    Emergency,
    /// Graceful is user initiated and expected.
    Graceful,
}

/// `err` decides, whether you should stream stderr or stdout.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct StreamRequestMessage {
    pub task_id: Option<usize>,
    pub err: bool,
    pub lines: Option<usize>,
}

/// Request logs for specific tasks.
///
/// `task_ids` specifies the requested tasks. If none are given, all tasks are selected.
/// `send_logs` Determines whether tasks should be sent at all.
/// `lines` Determines whether only a few lines of log should be returned.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct LogRequestMessage {
    pub task_ids: Vec<usize>,
    pub send_logs: bool,
    pub lines: Option<usize>,
}

/// Helper struct for sending tasks and their log output to the client.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TaskLogMessage {
    pub task: Task,
    pub stdout: Option<Vec<u8>>,
    pub stderr: Option<Vec<u8>>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct ParallelMessage {
    pub parallel_tasks: usize,
    pub group: String,
}

pub fn create_success_message<T: ToString>(text: T) -> Message {
    Message::Success(text.to_string())
}

pub fn create_failure_message<T: ToString>(text: T) -> Message {
    Message::Failure(text.to_string())
}
