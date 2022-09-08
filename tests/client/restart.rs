use std::collections::HashMap;

use anyhow::{bail, Result};
use pueue_lib::task::{TaskResult, TaskStatus};

use crate::fixtures::*;
use crate::helper::*;

/// Test that restarting a task while editing its command work as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restart_and_edit_task_command() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a task and wait for it to finish.
    assert_success(add_task(shared, "ls", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    // Set the editor to a command which replaces the temporary file's content.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "echo 'sleep 60' > ");

    // Restart the command, edit its command and wait for it to start.
    run_client_command_with_env(shared, &["restart", "--in-place", "--edit", "0"], envs).await?;
    wait_for_task_condition(shared, 0, |task| task.is_running()).await?;

    // Make sure that both the command has been updated.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.command, "sleep 60");
    assert_eq!(task.status, TaskStatus::Running);

    Ok(())
}

/// Test that restarting a task while editing its path work as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restart_and_edit_task_path() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a task and wait for it to finish.
    assert_success(add_task(shared, "ls", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    // Set the editor to a command which replaces the temporary file's content.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "echo '/tmp' > ");

    // Restart the command, edit its command and wait for it to start.
    run_client_command_with_env(shared, &["restart", "--in-place", "--edit-path", "0"], envs)
        .await?;
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    // Make sure that both the path has been updated.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.path.to_string_lossy(), "/tmp");

    Ok(())
}

/// Test that restarting a task while editing both, its command and its path, work as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restart_and_edit_task_path_and_command() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a task and wait for it to finish.
    assert_success(add_task(shared, "ls", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    // Set the editor to a command which replaces the temporary file's content.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "echo 'replaced string' > ");

    // Restart the command, edit its command and path and wait for it to start.
    // The task will fail afterwards, but it should still be edited.
    let output = run_client_command_with_env(
        shared,
        &[
            "restart",
            "--in-place",
            "--edit",
            "--edit-path",
            "--edit-label",
            "0",
        ],
        envs,
    )
    .await?;
    dbg!(output);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    // Make sure that both the path has been updated.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.command, "replaced string");
    assert_eq!(task.path.to_string_lossy(), "replaced string");
    assert_eq!(task.label, Some("replaced string".to_owned()));

    // Also the task should have been restarted and failed.
    if let TaskStatus::Done(TaskResult::FailedToSpawn(_)) = task.status {
    } else {
        bail!("The task should have failed");
    };

    Ok(())
}
