pub mod handler;

use ::chrono::prelude::*;
use ::chrono::DateTime;
use ::serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TaskStatus {
    Queued,
    Stashed,
    Running,
    Done,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: i32,
    pub command: String,
    pub arguments: Vec<String>,
    pub path: String,
    pub status: TaskStatus,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub start: Option<DateTime<Local>>,
    pub end: Option<DateTime<Local>>,
}

impl Task {
    pub fn new(command: String, arguments: Vec<String>, path: String) -> Task {
        Task {
            id: 0,
            command: command,
            arguments: arguments,
            path: path,
            status: TaskStatus::Queued,
            exit_code: None,
            stdout: None,
            stderr: None,
            start: None,
            end: None,
        }
    }
}
