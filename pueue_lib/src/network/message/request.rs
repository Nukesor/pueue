use std::{collections::HashMap, path::PathBuf};

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantNames};

use crate::network::message::EditableTask;

/// Macro to simplify creating [From] implementations for each variant-contained
/// Request; e.g. `impl_into_request!(AddRequest, Request::Add)` to make it possible
/// to use `AddRequest::into()` and get a [Request::Add] value.
macro_rules! impl_into_request {
    ($inner:ident, $variant:expr) => {
        impl From<$inner> for Request {
            fn from(message: $inner) -> Self {
                $variant(message)
            }
        }
    };
}

/// This is the message for messages sent **to** the daemon. \
/// Everything that's send by the client is represented using by this enum.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum Request {
    /// Add a new task to the daemon.
    Add(AddRequest),
    /// Remove non-running/paused tasks.
    Remove(Vec<usize>),
    /// Switch two enqueued/stashed tasks.
    Switch(SwitchRequest),
    /// Stash a task or schedule it for enqueue.
    Stash(StashRequest),
    /// Take a stashed task and enqueue it.
    Enqueue(EnqueueRequest),

    /// Start/unpause a [`TaskSelection`].
    Start(StartRequest),
    /// Restart a set of finished or failed task.
    Restart(RestartRequest),
    /// Pause a [`TaskSelection`].
    Pause(PauseRequest),
    /// Kill a [`TaskSelection`].
    Kill(KillRequest),

    /// Used to send some input to a process's stdin
    Send(SendRequest),

    /// The first part of the three-step protocol to edit a task.
    /// This one requests an edit from the daemon.
    EditRequest(Vec<usize>),
    /// This is send by the client if something went wrong during the editing process.
    /// The daemon will go ahead and restore the task's old state.
    EditRestore(Vec<usize>),
    /// The client sends the edited details to the daemon.
    EditedTasks(Vec<EditableTask>),

    /// Un/-set environment variables for specific tasks.
    Env(EnvRequest),

    Group(GroupRequest),

    /// Used to set parallel tasks for a specific group
    Parallel(ParallelRequest),

    /// Request the daemon's state
    Status,
    /// Request logs of a set of tasks.
    Log(LogRequest),

    /// The client requests a continuous stream of a task's log.
    Stream(StreamRequest),

    /// Reset the daemon
    Reset(ResetRequest),
    /// Tell the daemon to clean finished tasks
    Clean(CleanRequest),
    /// Initiate shutdown on the daemon.
    DaemonShutdown(ShutdownRequest),
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
pub struct AddRequest {
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
}

/// We use a custom `Debug` implementation for [AddRequest], as the `envs` field just has
/// too much info in it and makes the log output much too verbose.
///
/// Furthermore, there might be secrets in the environment, resulting in a possible leak
/// if users copy-paste their log output for debugging.
impl std::fmt::Debug for AddRequest {
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
            .finish()
    }
}
impl_into_request!(AddRequest, Request::Add);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct SwitchRequest {
    pub task_id_1: usize,
    pub task_id_2: usize,
}
impl_into_request!(SwitchRequest, Request::Switch);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct StashRequest {
    pub tasks: TaskSelection,
    pub enqueue_at: Option<DateTime<Local>>,
}
impl_into_request!(StashRequest, Request::Stash);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct EnqueueRequest {
    pub tasks: TaskSelection,
    pub enqueue_at: Option<DateTime<Local>>,
}
impl_into_request!(EnqueueRequest, Request::Enqueue);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct StartRequest {
    pub tasks: TaskSelection,
}
impl_into_request!(StartRequest, Request::Start);

/// The messages used to restart tasks.
/// It's possible to update the command and paths when restarting tasks.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct RestartRequest {
    pub tasks: Vec<TaskToRestart>,
    pub start_immediately: bool,
    pub stashed: bool,
}
impl_into_request!(RestartRequest, Request::Restart);

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
pub struct PauseRequest {
    pub tasks: TaskSelection,
    pub wait: bool,
}
impl_into_request!(PauseRequest, Request::Pause);

/// This is a small custom Enum for all currently supported unix signals.
/// Supporting all unix signals would be a mess, since there is a LOT of them.
///
/// This is also needed for usage in clap, since nix's Signal doesn't implement [Display] and
/// [std::str::FromStr].
#[derive(
    PartialEq, Eq, Clone, Debug, Deserialize, Serialize, Display, EnumString, VariantNames,
)]
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
pub struct KillRequest {
    pub tasks: TaskSelection,
    pub signal: Option<Signal>,
}
impl_into_request!(KillRequest, Request::Kill);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct SendRequest {
    pub task_id: usize,
    pub input: String,
}
impl_into_request!(SendRequest, Request::Send);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum EnvRequest {
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
impl_into_request!(EnvRequest, Request::Env);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum GroupRequest {
    Add {
        name: String,
        parallel_tasks: Option<usize>,
    },
    Remove(String),
    List,
}
impl_into_request!(GroupRequest, Request::Group);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum ResetTarget {
    // Reset all groups
    All,
    // Reset a list of specific groups
    Groups(Vec<String>),
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct ResetRequest {
    pub target: ResetTarget,
}
impl_into_request!(ResetRequest, Request::Reset);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct CleanRequest {
    pub successful_only: bool,

    pub group: Option<String>,
}
impl_into_request!(CleanRequest, Request::Clean);

/// Determines which type of shutdown we're dealing with.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum ShutdownRequest {
    /// Emergency is most likely a system unix signal or a CTRL+C in a terminal.
    Emergency,
    /// Graceful is user initiated and expected.
    Graceful,
}
impl_into_request!(ShutdownRequest, Request::DaemonShutdown);

/// Request the live streaming of a set of running tasks.
///
/// **WARNING**:
/// Even though this type currently accepts a TaskSelection, only
/// `TaskSelection::TaskIds(vec![])` and `TaskSelection::TaskIds(vec![id])` are accepted.
/// We already use this format in preparation for <https://github.com/Nukesor/pueue/issues/614>
/// That way we can stay forwards compatible without having to break the API.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct StreamRequest {
    pub tasks: TaskSelection,
    pub lines: Option<usize>,
}
impl_into_request!(StreamRequest, Request::Stream);

/// Request logs for specific tasks.
///
/// `tasks` specifies the requested tasks.
/// `send_logs` Determines whether logs should be sent at all.
/// `lines` Determines whether only a few lines of log should be returned.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct LogRequest {
    pub tasks: TaskSelection,
    pub send_logs: bool,
    pub lines: Option<usize>,
}
impl_into_request!(LogRequest, Request::Log);

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct ParallelRequest {
    pub parallel_tasks: usize,
    pub group: String,
}
impl_into_request!(ParallelRequest, Request::Parallel);
