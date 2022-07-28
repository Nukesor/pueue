use std::collections::HashMap;

use anyhow::{Context, Result};
use pueue_lib::network::message::Message;
use pueue_lib::task::TaskStatus;

use crate::fixtures::*;
use crate::helper::*;

/// Test that editing a task's path and command work as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn edit_task() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a stashed message which we'll edit later on.
    let mut message = create_add_message(shared, "this is a test");
    message.stashed = true;
    send_message(shared, Message::Add(message))
        .await
        .context("Failed to to add task to group.")?;

    // Update the task's command by piping a string to the temporary file.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "echo 'expected command string' > ");
    run_client_command_with_env(shared, &["edit", "0"], envs).await?;

    // Update the task's path by piping a string to the temporary file.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "echo 'expected path string' > ");
    run_client_command_with_env(shared, &["edit", "--path", "0"], envs).await?;

    // Make sure that both the command and the path have been updated.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.command, "expected command string");
    assert_eq!(task.path.to_string_lossy(), "expected path string");

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
        .context("Failed to to add task to group.")?;

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
