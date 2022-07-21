use std::collections::HashMap;

use anyhow::{Context, Result};
use pueue_lib::network::message::*;

use crate::fixtures::*;
use crate::helper::*;

/// Test that adding a group and getting the group overview works.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn default_group() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Add a group via the cli interface.
    run_client_command(shared, &["group", "add", "testgroup", "--parallel=2"]).await?;

    // Get the group status output
    let output = run_client_command(shared, &["group"]).await?;
    assert_stdout_matches("group__default_group", output.stdout, HashMap::new())?;

    Ok(())
}

/// Test that adding a group and getting the group overview with the `--color=always` flag works.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn colored_group() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Add a group via the cli interface.
    run_client_command(shared, &["group", "add", "testgroup", "--parallel=2"]).await?;

    // Pauses the default queue while waiting for tasks
    // We do this to ensure that paused groups are properly colored.
    let message = Message::Pause(PauseMessage {
        tasks: TaskSelection::Group(PUEUE_DEFAULT_GROUP.into()),
        wait: true,
        children: false,
    });
    send_message(shared, message)
        .await
        .context("Failed to send message")?;

    // Get the group status output
    let output = run_client_command(shared, &["--color", "always", "group"]).await?;
    assert_stdout_matches("group__group_with_color", output.stdout, HashMap::new())?;

    Ok(())
}
