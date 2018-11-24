use chrono::prelude::*;
use chrono::DateTime;

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub command: String,
    pub path: String,
    pub status: TaskStatus,
    pub returncode: Option<u16>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub start: Option<DateTime<Local>>,
    pub end: Option<DateTime<Local>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TaskStatus {
    Queued,
    Stashed,
    Running,
    Done,
    Failed,
}
