use std::io::{self, prelude::*};

use anyhow::{Context, Result};

use pueue_lib::{settings::Settings, state::State};

use crate::{
    cli::SubCommand,
    display::{colors::Colors, print_state},
};

/// This function tries to read a JSON serialized [State] from `stdin`.
/// The JSON document will then get deserialized and displayed as usual.
pub async fn format_state(
    group: &Option<String>,
    colors: &Colors,
    settings: &Settings,
) -> Result<()> {
    let mut stdin = io::stdin();
    let mut buffer = Vec::new();
    stdin
        .read_to_end(&mut buffer)
        .context("Failed to read json from stdin.")?;

    let json = String::from_utf8(buffer).context("Failed to convert stdin input to UTF8")?;
    let state: State =
        serde_json::from_str(&json).context("Failed to deserialize State from JSON input.")?;

    print_state(
        state,
        &SubCommand::FormatStatus {
            group: group.clone(),
        },
        colors,
        settings,
    );

    Ok(())
}
