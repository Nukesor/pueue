use chrono::prelude::*;
use chrono::DateTime;

pub struct Task {
    pub command: String,
    pub status: TaskStatus,
    pub returncode: Option<u16>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub start: Option<DateTime<Local>>,
    pub end: Option<DateTime<Local>>,
}

pub enum TaskStatus {
    Queued,
    Stashed,
    Running,
    Done,
    Failed,
}
