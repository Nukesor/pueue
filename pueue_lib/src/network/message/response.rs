use std::collections::BTreeMap;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    network::message::EditableTask,
    state::{Group, State},
    task::Task,
};

/// Macro to simplify creating success_messages
#[macro_export]
macro_rules! success_msg {
    ($($arg:tt)*) => {{
        create_success_response(format!($($arg)*))
    }}
}

/// Macro to simplify creating failure_messages
#[macro_export]
macro_rules! failure_msg {
    ($($arg:tt)*) => {{
        create_failure_response(format!($($arg)*))
    }}
}

pub fn create_success_response<T: ToString>(text: T) -> Response {
    Response::Success(text.to_string())
}

pub fn create_failure_response<T: ToString>(text: T) -> Response {
    Response::Failure(text.to_string())
}

/// Macro to simplify creating [From] implementations for each variant-contained
/// Response; e.g. `impl_into_response!(AddRequest, Response::Add)` to implement
/// use `AddedTaskResponse::into()` and get a [Response::AddedTask] value.
macro_rules! impl_into_response {
    ($inner:ty, $variant:expr) => {
        impl From<$inner> for Response {
            fn from(message: $inner) -> Self {
                $variant(message)
            }
        }
    };
}

/// This is the message for messages sent **to** the daemon. \
/// Everything that's send by the client is represented using by this enum.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum Response {
    AddedTask(AddedTaskResponse),

    /// The daemon locked the tasks and responds with the tasks' details.
    Edit(Vec<EditableTask>),

    Status(Box<State>),

    /// The log returned from the daemon for a bunch of [`Task`]s
    /// This is the response to [`super::Request::Log`]
    Log(BTreeMap<usize, TaskLogResponse>),

    Group(GroupResponse),

    /// The next chunk of output, that's send to the client.
    Stream(StreamResponse),

    Success(String),
    Failure(String),

    /// Simply notify the client that the connection is now closed.
    /// This is used to, for instance, close a `follow` stream if the task finished.
    Close,
}

impl Response {
    pub fn success(&self) -> bool {
        matches!(&self, Self::AddedTask(_) | Self::Success(_))
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Default, Deserialize, Serialize)]
pub struct AddedTaskResponse {
    pub task_id: usize,
    pub enqueue_at: Option<DateTime<Local>>,
    pub group_is_paused: bool,
}
impl_into_response!(AddedTaskResponse, Response::AddedTask);

/// Helper struct for sending tasks and their log output to the client.
#[derive(PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct TaskLogResponse {
    pub task: Task,
    /// Indicates whether the log output has been truncated or not.
    pub output_complete: bool,
    pub output: Option<Vec<u8>>,
}
impl_into_response!(BTreeMap<usize, TaskLogResponse>, Response::Log);

/// We use a custom `Debug` implementation for [TaskLogResponse], as the `output` field
/// has too much info in it and renders log output unreadable.
impl std::fmt::Debug for TaskLogResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskLogResponse")
            .field("task", &self.task)
            .field("output_complete", &self.output_complete)
            .field("output", &"hidden")
            .finish()
    }
}

/// Group info send by the daemon.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct GroupResponse {
    pub groups: BTreeMap<String, Group>,
}
impl_into_response!(GroupResponse, Response::Group);

/// Live log output returned by the daemon.
///
/// The logs are ordered by task id.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct StreamResponse {
    pub logs: BTreeMap<usize, String>,
}
impl_into_response!(StreamResponse, Response::Stream);
