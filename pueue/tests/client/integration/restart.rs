use std::collections::HashMap;

use anyhow::{bail, Result};
use pueue_lib::task::{TaskResult, TaskStatus};

use crate::client::helper::*;

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
    run_client_command_with_env(shared, &["restart", "--in-place", "--edit", "0"], envs)?;
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
    run_client_command_with_env(shared, &["restart", "--in-place", "--edit-path", "0"], envs)?;
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
    assert_success(add_task(shared, "ls", false).await.unwrap());
    wait_for_task_condition(shared, 0, |task| task.is_done())
        .await
        .unwrap();

    // Set the editor to a command which replaces the temporary file's content.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "echo 'replaced string' > ");

    // Restart the command, edit its command and path and wait for it to start.
    // The task will fail afterwards, but it should still be edited.
    run_client_command_with_env(
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
    )?;
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

/// Test that restarting a task **not** in place works as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn normal_restart_with_edit() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a task and wait for it to finish.
    assert_success(add_task(shared, "ls", false).await?);
    let original_task = wait_for_task_condition(shared, 0, |task| task.is_done()).await?;
    assert!(
        original_task.enqueued_at.is_some(),
        "Task is done and should have enqueue_at set."
    );

    // Set the editor to a command which replaces the temporary file's content.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "echo 'sleep 60' > ");

    // Restart the command, edit its command and wait for it to start.
    run_client_command_with_env(shared, &["restart", "--edit", "0"], envs)?;
    wait_for_task_condition(shared, 1, |task| task.is_running()).await?;

    // Make sure that both the command has been updated.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&1).unwrap();
    assert_eq!(task.command, "sleep 60");
    assert_eq!(task.status, TaskStatus::Running);

    // Since we created a copy, the new task should be created after the first one.
    assert!(
        original_task.created_at < task.created_at,
        "New task should have a newer created_at."
    );
    // The created_at time should also be newer.
    assert!(
        original_task.enqueued_at.unwrap() < task.enqueued_at.unwrap(),
        "The second run should be enqueued before the first run."
    );

    Ok(())
}
