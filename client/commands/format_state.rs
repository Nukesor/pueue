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

/// This function tries to read a list of JSON serialized [Task]s from `stdin`.
/// The tasks will then get deserialized and displayed as a normal `status` command.
/// The current group information is pulled from the daemon in a new `status` call.
pub async fn format_state(
    stream: &mut GenericStream,
    command: &SubCommand,
    colors: &Colors,
    settings: &Settings,
) -> Result<()> {
    let mut stdin = io::stdin();
    let mut buffer = Vec::new();
    stdin
        .read_to_end(&mut buffer)
        .context("Failed to read json from stdin.")?;

    // Get the
    let json = String::from_utf8(buffer).context("Failed to convert stdin input to UTF8")?;
    let tasks: BTreeMap<usize, Task> =
        serde_json::from_str(&json).context("Failed to deserialize State from JSON input.")?;

    let mut state = super::get_state(stream)
        .await
        .context("Failed to get the current state from daemon")?;

    state.tasks = tasks;

    print_state(state, command, colors, settings);

    Ok(())
}
