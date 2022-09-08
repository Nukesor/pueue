use std::collections::HashMap;

use anyhow::{Context, Result};
use pueue_lib::network::message::Message;
use pueue_lib::task::TaskStatus;

use crate::fixtures::*;
use crate::helper::*;

/// Test that editing a task without any flags only updates the command.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn edit_task_default() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a stashed message which we'll edit later on.
    let mut message = create_add_message(shared, "this is a test");
    message.stashed = true;
    send_message(shared, Message::Add(message))
        .await
        .context("Failed to to add stashed task.")?;

    // Update the task's command by piping a string to the temporary file.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "echo 'expected command string' > ");
    run_client_command_with_env(shared, &["edit", "0"], envs).await?;

    // Make sure that both the command has been updated.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.command, "expected command string");

    // All other properties should be unchanged.
    assert_eq!(task.path, daemon.tempdir.path());
    assert_eq!(task.label, None);

    Ok(())
}

/// Test that editing a task's path and command work as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn edit_all_task_properties() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a stashed message which we'll edit later on.
    let mut message = create_add_message(shared, "this is a test");
    message.stashed = true;
    send_message(shared, Message::Add(message))
        .await
        .context("Failed to to add stashed task.")?;

    // Update all task properties by piping a string to the respective temporary file.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "echo 'expected string' > ");
    run_client_command_with_env(
        shared,
        &["edit", "--command", "--path", "--label", "0"],
        envs,
    )
    .await?;

    // Make sure that all properties have been updated.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.command, "expected string");
    assert_eq!(task.path.to_string_lossy(), "expected string");
    assert_eq!(task.label, Some("expected string".to_string()));

    Ok(())
}

/// Test that automatic restoration of a task's state works, if the edit command fails for some
/// reason.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fail_to_edit_task() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a stashed message which we'll edit later on.
    let mut message = create_add_message(shared, "this is a test");
    message.stashed = true;
    send_message(shared, Message::Add(message))
        .await
        .context("Failed to to add stashed task.")?;

    // Run a editor command that crashes.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "non_existing_test_binary");
    let command = run_client_command_with_env(shared, &["edit", "0"], envs).await;
    assert!(
        command.is_err(),
        "The command should fail, as the command isn't valid"
    );

    // Make sure that nothing has changed and the task is `Stashed` again.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.command, "this is a test");
    assert_eq!(task.status, TaskStatus::Stashed { enqueue_at: None });

    Ok(())
}
