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
