use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::state::{Group, State};
use crate::task::Task;

/// Macro to simplify creating [From] implementations for each variant-contained
/// struct; e.g. `impl_into_message!(AddMessage, Message::Add)` to make it possible
/// to use `AddMessage { }.into()` and get a `Message::Add()` value.
macro_rules! impl_into_message {
    ($inner:ident, $variant:expr) => {
        impl From<$inner> for Message {
            fn from(message: $inner) -> Self {
                $variant(message)
            }
        }
    };
}

/// Macro to simplify creating success_messages
#[macro_export]
macro_rules! success_msg {
    ($($arg:tt)*) => {{
        create_success_message(format!($($arg)*))
    }}
}

/// Macro to simplify creating failure_messages
#[macro_export]
macro_rules! failure_msg {
    ($($arg:tt)*) => {{
        create_failure_message(format!($($arg)*))
    }}
}

/// This is the main message enum. \
/// Everything that's send between the daemon and a client can be represented by this enum.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    Add(AddMessage),
    Remove(Vec<usize>),
    Switch(SwitchMessage),
    Stash(StashMessage),
    Enqueue(EnqueueMessage),

    Start(StartMessage),
    Restart(RestartMessage),
    Pause(PauseMessage),
    Kill(KillMessage),

    /// Used to send some input to a process's stdin
    Send(SendMessage),

    /// The first part of the three-step protocol to edit a task.
    /// This one requests an edit from the daemon.
    EditRequest(Vec<usize>),
    /// The daemon locked the tasks and responds with the tasks' details.
    EditResponse(Vec<EditableTask>),
    /// This is send by the client if something went wrong during the editing process.
    /// The daemon will go ahead and restore the task's old state.
    EditRestore(Vec<usize>),
    /// The client sends the edited details to the daemon.
    Edit(Vec<EditableTask>),

    Env(EnvMessage),

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

    Reset(ResetMessage),
    Clean(CleanMessage),
    DaemonShutdown(Shutdown),

    Success(String),
    Failure(String),
    /// Simply notify the client that the connection is now closed.
    /// This is used to, for instance, close a `follow` stream if the task finished.
    Close,

    Parallel(ParallelMessage),
}

/// This enum is used to express a selection of tasks.
/// As commands can be executed on various sets of tasks, we need some kind of datastructure to
/// explicitly and unambiguously specify the selection.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum TaskSelection {
    TaskIds(Vec<usize>),
    Group(String),
    All,
}

#[derive(PartialEq, Eq, Clone, Default, Deserialize, Serialize)]
pub struct AddMessage {
    pub command: String,
    pub path: PathBuf,
    pub envs: HashMap<String, String>,
    pub start_immediately: bool,
    pub stashed: bool,
    pub group: String,
    pub enqueue_at: Option<DateTime<Local>>,
    pub dependencies: Vec<usize>,
    pub priority: Option<i32>,
    pub label: Option<String>,
    pub print_task_id: bool,
}

/// We use a custom `Debug` implementation for [AddMessage], as the `envs` field just has
/// too much info in it and makes the log output much too verbose.
///
/// Furthermore, there might be secrets in the environment, resulting in a possible leak
/// if users copy-paste their log output for debugging.
impl std::fmt::Debug for AddMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("command", &self.command)
            .field("path", &self.path)
            .field("envs", &"hidden")
            .field("start_immediately", &self.start_immediately)
            .field("stashed", &self.stashed)
            .field("group", &self.group)
            .field("enqueue_at", &self.enqueue_at)
            .field("dependencies", &self.dependencies)
            .field("label", &self.label)
            .field("print_task_id", &self.print_task_id)
            .finish()
    }
}

impl_into_message!(AddMessage, Message::Add);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct SwitchMessage {
    pub task_id_1: usize,
    pub task_id_2: usize,
}

impl_into_message!(SwitchMessage, Message::Switch);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct StashMessage {
    pub tasks: TaskSelection,
    pub enqueue_at: Option<DateTime<Local>>,
}

impl_into_message!(StashMessage, Message::Stash);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct EnqueueMessage {
    pub tasks: TaskSelection,
    pub enqueue_at: Option<DateTime<Local>>,
}

impl_into_message!(EnqueueMessage, Message::Enqueue);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct StartMessage {
    pub tasks: TaskSelection,
}

impl_into_message!(StartMessage, Message::Start);

/// The messages used to restart tasks.
/// It's possible to update the command and paths when restarting tasks.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct RestartMessage {
    pub tasks: Vec<TaskToRestart>,
    pub start_immediately: bool,
    pub stashed: bool,
}

impl_into_message!(RestartMessage, Message::Restart);

#[derive(PartialEq, Eq, Clone, Debug, Default, Deserialize, Serialize)]
pub struct TaskToRestart {
    pub task_id: usize,
    /// Restart the task with an updated command.
    pub command: String,
    /// Restart the task with an updated path.
    pub path: PathBuf,
    /// Restart the task with an updated label.
    pub label: Option<String>,
    /// Restart the task with an updated priority.
    pub priority: i32,
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct PauseMessage {
    pub tasks: TaskSelection,
    pub wait: bool,
}

impl_into_message!(PauseMessage, Message::Pause);

/// This is a small custom Enum for all currently supported unix signals.
/// Supporting all unix signals would be a mess, since there is a LOT of them.
///
/// This is also needed for usage in clap, since nix's Signal doesn't implement [Display] and
/// [std::str::FromStr].
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize, Display, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Signal {
    #[strum(serialize = "sigint", serialize = "int", serialize = "2")]
    SigInt,
    #[strum(serialize = "sigkill", serialize = "kill", serialize = "9")]
    SigKill,
    #[strum(serialize = "sigterm", serialize = "term", serialize = "15")]
    SigTerm,
    #[strum(serialize = "sigcont", serialize = "cont", serialize = "18")]
    SigCont,
    #[strum(serialize = "sigstop", serialize = "stop", serialize = "19")]
    SigStop,
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct KillMessage {
    pub tasks: TaskSelection,
    pub signal: Option<Signal>,
}

impl_into_message!(KillMessage, Message::Kill);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct SendMessage {
    pub task_id: usize,
    pub input: String,
}

impl_into_message!(SendMessage, Message::Send);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct EditableTask {
    pub id: usize,
    pub command: String,
    pub path: PathBuf,
    pub label: Option<String>,
    pub priority: i32,
}

impl From<&Task> for EditableTask {
    /// Create an editable tasks from any [Task]]
    fn from(value: &Task) -> Self {
        EditableTask {
            id: value.id,
            command: value.command.clone(),
            path: value.path.clone(),
            label: value.label.clone(),
            priority: value.priority,
        }
    }
}

impl EditableTask {
    /// Merge a [EditableTask] back into a [Task].
    pub fn into_task(self, task: &mut Task) {
        task.command = self.command;
        task.path = self.path;
        task.label = self.label;
        task.priority = self.priority;
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum EnvMessage {
    Set {
        task_id: usize,
        key: String,
        value: String,
    },
    Unset {
        task_id: usize,
        key: String,
    },
}

impl_into_message!(EnvMessage, Message::Env);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum GroupMessage {
    Add {
        name: String,
        parallel_tasks: Option<usize>,
    },
    Remove(String),
    List,
}

impl_into_message!(GroupMessage, Message::Group);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct GroupResponseMessage {
    pub groups: BTreeMap<String, Group>,
}

impl_into_message!(GroupResponseMessage, Message::GroupResponse);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum ResetTarget {
    // Reset all groups
    All,
    // Reset a list of specific groups
    Groups(Vec<String>),
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct ResetMessage {
    pub target: ResetTarget,
}

impl_into_message!(ResetMessage, Message::Reset);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct CleanMessage {
    pub successful_only: bool,

    pub group: Option<String>,
}

impl_into_message!(CleanMessage, Message::Clean);

/// Determines which type of shutdown we're dealing with.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum Shutdown {
    /// Emergency is most likely a system unix signal or a CTRL+C in a terminal.
    Emergency,
    /// Graceful is user initiated and expected.
    Graceful,
}

impl_into_message!(Shutdown, Message::DaemonShutdown);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct StreamRequestMessage {
    pub task_id: Option<usize>,
    pub lines: Option<usize>,
}

impl_into_message!(StreamRequestMessage, Message::StreamRequest);

/// Request logs for specific tasks.
///
/// `tasks` specifies the requested tasks.
/// `send_logs` Determines whether logs should be sent at all.
/// `lines` Determines whether only a few lines of log should be returned.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct LogRequestMessage {
    pub tasks: TaskSelection,
    pub send_logs: bool,
    pub lines: Option<usize>,
}

impl_into_message!(LogRequestMessage, Message::Log);

/// Helper struct for sending tasks and their log output to the client.
#[derive(PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct TaskLogMessage {
    pub task: Task,
    /// Indicates whether the log output has been truncated or not.
    pub output_complete: bool,
    pub output: Option<Vec<u8>>,
}

/// We use a custom `Debug` implementation for [TaskLogMessage], as the `output` field
/// has too much info in it and renders log output unreadable.
impl std::fmt::Debug for TaskLogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskLogMessage")
            .field("task", &self.task)
            .field("output_complete", &self.output_complete)
            .field("output", &"hidden")
            .finish()
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct ParallelMessage {
    pub parallel_tasks: usize,
    pub group: String,
}

impl_into_message!(ParallelMessage, Message::Parallel);

pub fn create_success_message<T: ToString>(text: T) -> Message {
    Message::Success(text.to_string())
}

pub fn create_failure_message<T: ToString>(text: T) -> Message {
    Message::Failure(text.to_string())
}
