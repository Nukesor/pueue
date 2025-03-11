//! This contains the the [`Request`] and [`Response`]  enums and
//! all their structs used to communicate with the daemon or client.
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::task::Task;

pub mod request;
pub mod response;

pub use request::*;
pub use response::*;

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct EditableTask {
    pub id: usize,
    #[serde(rename = "command")]
    pub original_command: String,
    pub path: PathBuf,
    pub label: Option<String>,
    pub priority: i32,
}

impl From<&Task> for EditableTask {
    /// Create an editable tasks from any [Task]]
    fn from(task: &Task) -> Self {
        EditableTask {
            id: task.id,
            original_command: task.original_command.clone(),
            path: task.path.clone(),
            label: task.label.clone(),
            priority: task.priority,
        }
    }
}

impl EditableTask {
    /// Merge a [EditableTask] back into a [Task].
    pub fn into_task(self, task: &mut Task) {
        task.original_command = self.original_command;
        task.path = self.path;
        task.label = self.label;
        task.priority = self.priority;
    }
}
