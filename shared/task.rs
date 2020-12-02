use std::collections::HashMap;

use chrono::prelude::*;
use serde_derive::{Deserialize, Serialize};
use strum_macros::Display;

use crate::aliasing::insert_alias;

/// This enum represents the status of the internal task handling of Pueue.
/// They basically represent the internal task life-cycle.
#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// The task is queued and waiting for a free slot
    Queued,
    /// The task has been manually stashed. It won't be executed until it's manually enqueued
    Stashed,
    /// The task is started and running
    Running,
    /// A previously running task has been paused
    Paused,
    /// Task finished. The actual result of the task is handled by the [TaskResult] enum.
    Done,
    /// Used while the command of a task is edited (to prevent starting the task)
    Locked,
}

/// This enum represents the exit status of an actually spawned program.
/// It's only used, once a task finished or failed in some kind of way.
#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
pub enum TaskResult {
    /// Task exited with 0
    Success,
    /// The task failed in some other kind of way (error code != 0)
    Failed(i32),
    /// The task couldn't be spawned. Probably a typo in the command
    FailedToSpawn(String),
    /// Task has been actively killed by either the user or the daemon on shutdown
    Killed,
    /// A dependency of the task failed.
    DependencyFailed,
}

/// Representation of a task.
/// start will be set the second the task starts processing.
/// exit_code, output and end won't be initialized, until the task has finished.
/// The output of the task is written into seperate files.
/// Upon task completion, the output is read from the files and put into the struct.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Task {
    pub id: usize,
    pub original_command: String,
    pub command: String,
    pub path: String,
    pub envs: HashMap<String, String>,
    pub group: Option<String>,
    pub enqueue_at: Option<DateTime<Local>>,
    pub dependencies: Vec<usize>,
    pub status: TaskStatus,
    pub prev_status: TaskStatus,
    pub result: Option<TaskResult>,
    pub start: Option<DateTime<Local>>,
    pub end: Option<DateTime<Local>>,
}

impl Task {
    pub fn new(
        original_command: String,
        path: String,
        envs: HashMap<String, String>,
        group: Option<String>,
        starting_status: TaskStatus,
        enqueue_at: Option<DateTime<Local>>,
        dependencies: Vec<usize>,
    ) -> Task {
        let command = insert_alias(original_command.clone());

        Task {
            id: 0,
            original_command,
            command,
            path,
            envs,
            group,
            enqueue_at,
            dependencies,
            status: starting_status.clone(),
            prev_status: starting_status,
            result: None,
            start: None,
            end: None,
        }
    }

    /// A convenience function, which is used to create a duplicate task.
    pub fn from_task(task: &Task) -> Task {
        Task {
            id: 0,
            original_command: task.original_command.clone(),
            command: task.command.clone(),
            path: task.path.clone(),
            envs: task.envs.clone(),
            group: None,
            enqueue_at: None,
            dependencies: Vec::new(),
            status: TaskStatus::Queued,
            prev_status: TaskStatus::Queued,
            result: None,
            start: None,
            end: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.status == TaskStatus::Running || self.status == TaskStatus::Paused
    }

    pub fn is_done(&self) -> bool {
        self.status == TaskStatus::Done
    }

    /// Check if the task errored.
    /// The only case when it didn't error is if it didn't run yet or if the task exited successfully.
    pub fn failed(&self) -> bool {
        !matches!(self.result, None | Some(TaskResult::Success))
    }

    pub fn is_queued(&self) -> bool {
        self.status == TaskStatus::Queued || self.status == TaskStatus::Stashed
    }
}
