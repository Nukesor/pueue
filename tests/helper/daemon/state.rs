use anyhow::{anyhow, bail, Result};

use pueue_lib::network::message::*;
use pueue_lib::settings::*;
use pueue_lib::state::State;
use pueue_lib::task::{Task, TaskStatus};

use super::*;

/// Convenience function for getting the current state from the daemon.
pub async fn get_state(shared: &Shared) -> Result<Box<State>> {
    let response = send_message(shared, Message::Status).await?;
    match response {
        Message::StatusResponse(state) => Ok(state),
        _ => bail!("Didn't get status response in get_state"),
    }
}

/// Convenience wrapper around `get_state` if you only need a single task.
pub async fn get_task(shared: &Shared, task_id: usize) -> Result<Task> {
    let state = get_state(shared).await?;
    let task = state
        .tasks
        .get(&0)
        .ok_or(anyhow!("Couldn't find task {}", task_id))?;

    Ok(task.clone())
}

/// Convenience wrapper around `get_task` if you really only need the task's status.
pub async fn get_task_status(shared: &Shared, task_id: usize) -> Result<TaskStatus> {
    let task = get_task(shared, task_id).await?;
    Ok(task.status)
}
