use std::{
    collections::BTreeMap,
    io::{self, prelude::*},
};

use anyhow::{Context, Result};

use pueue_lib::{network::protocol::GenericStream, settings::Settings, task::Task};

use crate::{
    cli::SubCommand,
    display::{colors::Colors, print_state},
};

/// This function tries to read a map or list of JSON serialized [Task]s from `stdin`.
/// The tasks will then get deserialized and displayed as a normal `status` command.
/// The current group information is pulled from the daemon in a new `status` call.
pub async fn format_state(
    stream: &mut GenericStream,
    command: &SubCommand,
    colors: &Colors,
    settings: &Settings,
) -> Result<()> {
    // Read the raw input to a buffer
    let mut stdin = io::stdin();
    let mut buffer = Vec::new();
    stdin
        .read_to_end(&mut buffer)
        .context("Failed to read json from stdin.")?;

    // Convert it to a valid utf8 stream. If this fails, it cannot be valid JSON.
    let json = String::from_utf8(buffer).context("Failed to convert stdin input to UTF8")?;

    // Try to deserialize the input as a map of tasks first.
    // If this doesn't work, try a list of tasks.
    let map_deserialize = serde_json::from_str::<BTreeMap<usize, Task>>(&json);
    let tasks: BTreeMap<usize, Task> = match map_deserialize {
        Ok(tasks) => tasks,
        Err(_) => {
            let task_list: Vec<Task> =
                serde_json::from_str(&json).context("Failed to deserialize from JSON input.")?;
            task_list.into_iter().map(|task| (task.id, task)).collect()
        }
    };

    let mut state = super::get_state(stream)
        .await
        .context("Failed to get the current state from daemon")?;

    state.tasks = tasks;

    print_state(state, command, colors, settings);

    Ok(())
}
