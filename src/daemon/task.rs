use chrono::prelude::*;
use chrono::DateTime;

pub struct Task {
    status: TaskStatus,
    returncode: Option<u16>,
    stdout: String,
    stderr: String,
    start: Option<DateTime<Local>>,
    end: Option<DateTime<Local>>,
}

pub enum TaskStatus {
    Queued,
    Stashed,
    Running,
    Done,
    Failed,
}
