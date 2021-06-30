use std::collections::HashMap;

use chrono::prelude::*;
use serde::{Deserialize, Deserializer};
use serde_derive::{Deserialize, Serialize};
use strum_macros::Display;

use crate::aliasing::insert_alias;

/// This enum represents the status of the internal task handling of Pueue.
/// They basically represent the internal task life-cycle.
#[derive(PartialEq, Clone, Debug, Display, Serialize, Deserialize)]
pub enum TaskStatus {
    /// The task is queued and waiting for a free slot
    Queued,
    /// The task has been manually stashed. It won't be executed until it's manually enqueued
    #[serde(deserialize_with = "enqueue_at_or_default")]
    Stashed { enqueue_at: Option<DateTime<Local>> },
    /// The task is started and running
    Running,
    /// A previously running task has been paused
    Paused,
    /// Task finished. The actual result of the task is handled by the [TaskResult] enum.
    Done(TaskResult),
    /// Used while the command of a task is edited (to prevent starting the task)
    Locked,
}

fn enqueue_at_or_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + Default,
    D: Deserializer<'de>,
{
    let v: Value = Deserialize::deserialize(deserializer)?;
    Ok(T::deserialize(v).unwrap_or_default())
}

/// This enum represents the exit status of an actually spawned program.
/// It's only used, once a task finished or failed in some kind of way.
#[derive(PartialEq, Clone, Debug, Display, Serialize, Deserialize)]
pub enum TaskResult {
    /// Task exited with 0
    Success,
    /// The task failed in some other kind of way (error code != 0)
    Failed(i32),
    /// The task couldn't be spawned. Probably a typo in the command
    FailedToSpawn(String),
    /// Task has been actively killed by either the user or the daemon on shutdown
    Killed,
    /// Some kind of IO error. This should barely ever happen. Please check the daemon logs.
    Errored,
    /// A dependency of the task failed.
    DependencyFailed,
}

/// Representation of a task.
/// start will be set the second the task starts processing.
/// `result`, `output` and `end` won't be initialized, until the task has finished.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Task {
    pub id: usize,
    pub original_command: String,
    pub command: String,
    pub path: String,
    pub envs: HashMap<String, String>,
    pub group: String,
    pub dependencies: Vec<usize>,
    pub label: Option<String>,
    pub status: TaskStatus,
    /// This field is only used when editing the path/command of a task.
    /// It's necessary, since we enter the `Locked` state during editing.
    /// However, we have to go back to the previous state after we finished editing.
    pub prev_status: TaskStatus,
    pub start: Option<DateTime<Local>>,
    pub end: Option<DateTime<Local>>,
}

impl Task {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        original_command: String,
        path: String,
        envs: HashMap<String, String>,
        group: String,
        starting_status: TaskStatus,
        dependencies: Vec<usize>,
        label: Option<String>,
    ) -> Task {
        let command = insert_alias(original_command.clone());

        Task {
            id: 0,
            original_command,
            command,
            path,
            envs,
            group,
            dependencies,
            label,
            status: starting_status.clone(),
            prev_status: starting_status,
            start: None,
            end: None,
        }
    }

    /// A convenience function used to duplicate a task.
    pub fn from_task(task: &Task) -> Task {
        Task {
            id: 0,
            original_command: task.original_command.clone(),
            command: task.command.clone(),
            path: task.path.clone(),
            envs: task.envs.clone(),
            group: "default".to_string(),
            dependencies: Vec::new(),
            label: task.label.clone(),
            status: TaskStatus::Queued,
            prev_status: TaskStatus::Queued,
            start: None,
            end: None,
        }
    }

    /// Whether the task is having a running process managed by the TaskHandler
    pub fn is_running(&self) -> bool {
        self.status == TaskStatus::Running || self.status == TaskStatus::Paused
    }

    /// Whether the task's process finished.
    pub fn is_done(&self) -> bool {
        matches!(self.status, TaskStatus::Done(_))
    }

    /// Check if the task errored. \
    /// It either:
    /// 1. Finished successfully
    /// 2. Didn't finish yet.
    pub fn failed(&self) -> bool {
        matches!(self.status, TaskStatus::Done(TaskResult::Success))
            || !matches!(self.status, TaskStatus::Done(_))
    }

    pub fn is_queued(&self) -> bool {
        matches!(self.status, TaskStatus::Queued | TaskStatus::Stashed { .. })
    }

    /// Small convenience function to set the task's group to the default group.
    pub fn set_default_group(&mut self) {
        self.group = String::from("default");
    }

    pub fn is_in_default_group(&self) -> bool {
        self.group.eq("default")
    }
}
