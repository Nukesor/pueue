use std::{
    collections::BTreeMap,
    io::{self, prelude::*},
};

use pueue_lib::task::Task;

use crate::{
    client::{client::Client, commands::get_state, display::print_state},
    internal_prelude::*,
};

/// This function tries to read a map or list of JSON serialized [Task]s from `stdin`.
/// The tasks will then get deserialized and displayed as a normal `status` command.
/// The current group information is pulled from the daemon in a new `status` call.
pub async fn format_state(client: &mut Client) -> Result<()> {
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

    let tasks: Vec<Task> = if let Ok(map) = map_deserialize {
        map.into_values().collect()
    } else {
        serde_json::from_str(&json).context("Failed to deserialize from JSON input.")?
    };

    let state = get_state(client)
        .await
        .context("Failed to get the current state from daemon")?;

    let output = print_state(
        state,
        tasks,
        &client.subcommand,
        &client.style,
        &client.settings,
    )?;
    print!("{output}");

    Ok(())
}
