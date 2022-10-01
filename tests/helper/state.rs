use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use serde_derive::Deserialize;

use pueue_lib::network::message::*;
use pueue_lib::settings::*;
use pueue_lib::state::State;
use pueue_lib::task::Task;

use super::*;

#[derive(Deserialize)]
pub struct JsonTasks {
    tasks: HashMap<usize, Task>,
}

/// Convenience function for getting the current state from the daemon.
pub async fn get_state(shared: &Shared) -> Result<Box<State>> {
    let response = send_message(shared, Message::Status).await?;
    match response {
        Message::StatusResponse(state) => Ok(state),
        _ => bail!("Didn't get status response in get_state"),
    }
}

/// Convenience function for getting a list of tasks via `status --json` from the daemon.
pub async fn get_json_tasks_from_command(shared: &Shared, query: &[&str]) -> Result<Vec<Task>> {
    let mut args = vec!["status", "--json"];
    args.append(&mut query.to_owned());
    let output = run_client_command(shared, &args)
        .context(format!("Failed to run command with {args:?}"))?;

    let json = String::from_utf8_lossy(&output.stdout);

    let tasks: JsonTasks =
        serde_json::from_str(&json).context("Failed to deserialize json string")?;

    Ok(tasks.tasks.into_values().collect())
}
