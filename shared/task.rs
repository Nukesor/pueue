use ::chrono::prelude::*;
use ::serde_derive::{Deserialize, Serialize};
use ::strum_macros::{Display, EnumIter};

#[derive(Clone, Display, Debug, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum TaskStatus {
    /// The task is queued and waiting for a free slot
    Queued,
    /// The task has been manually stashed. It won't be executed until it's manually enqueued
    Stashed,
    /// The task is started and running
    Running,
    /// A previously running task has been paused
    Paused,
    /// Task finished successfully
    Done,
    /// The task failed in some other kind of way (error code != 0)
    Failed,
    /// The task couldn't be spawned. Probably a typo in the command
    FailedToSpawn,
    /// Task has been actively killed by either the user or the daemon on shutdown
    Killed,
    /// Used while the command of a task is edited (to prevent starting the task)
    Locked,
}

/// Representation of a task.
/// start will be set the second the task starts processing.
/// exit_code, output and end won't be initialized, until the task has finished.
/// The output of the task is written into seperate files.
/// Upon task completion, the output is read from the files and put into the struct.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: usize,
    pub command: String,
    pub path: String,
    pub enqueue_at: Option<DateTime<Local>>,
    pub status: TaskStatus,
    pub prev_status: TaskStatus,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub start: Option<DateTime<Local>>,
    pub end: Option<DateTime<Local>>,
    pub dependencies: Vec<usize>,
}

impl Task {
    pub fn new(
        command: String,
        path: String,
        starting_status: TaskStatus,
        enqueue_at: Option<DateTime<Local>>,
        dependencies: Vec<usize>,
    ) -> Task {
        Task {
            id: 0,
            command: command,
            path: path,
            enqueue_at: enqueue_at,
            status: starting_status.clone(),
            prev_status: starting_status,
            exit_code: None,
            stdout: None,
            stderr: None,
            start: None,
            end: None,
            dependencies,
        }
    }

    pub fn from_task(task: &Task) -> Task {
        Task {
            id: 0,
            command: task.command.clone(),
            path: task.path.clone(),
            enqueue_at: None,
            status: TaskStatus::Queued,
            prev_status: TaskStatus::Queued,
            exit_code: None,
            stdout: None,
            stderr: None,
            start: None,
            end: None,
            dependencies: task.dependencies.clone(),
        }
    }

    pub fn is_running(&self) -> bool {
        return self.status == TaskStatus::Running || self.status == TaskStatus::Paused;
    }

    pub fn is_done(&self) -> bool {
        return self.status == TaskStatus::Done
            || self.status == TaskStatus::Failed
            || self.status == TaskStatus::Killed;
    }

    pub fn is_errored(&self) -> bool {
        return self.status == TaskStatus::Failed || self.status == TaskStatus::Killed;
    }

    pub fn is_queued(&self) -> bool {
        return self.status == TaskStatus::Queued || self.status == TaskStatus::Stashed;
    }
}
