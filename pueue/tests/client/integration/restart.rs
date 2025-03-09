use std::collections::HashMap;

use assert_matches::assert_matches;
use color_eyre::eyre::ContextCompat;
use pueue_lib::{Task, TaskResult, TaskStatus};

use crate::{client::helper::*, internal_prelude::*};

/// Test that restarting a task while editing its command work as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restart_and_edit_task_command() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a task and wait for it to finish.
    assert_success(add_task(shared, "ls").await?);
    wait_for_task_condition(shared, 0, Task::is_done).await?;

    // Set the editor to a command which replaces the temporary file's content.
    let mut envs = HashMap::new();
    envs.insert(
        "EDITOR",
        "echo 'sleep 60' > ${PUEUE_EDIT_PATH}/0/command ||",
    );

    // Restart the command, edit its command and wait for it to start.
    run_client_command_with_env(shared, &["restart", "--in-place", "--edit", "0"], envs)?
        .success()?;
    wait_for_task_condition(shared, 0, Task::is_running).await?;

    // Make sure that the command has been updated.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.command, "sleep 60");
    assert_matches!(
        task.status,
        TaskStatus::Running { .. },
        "Task should be running"
    );

    Ok(())
}

/// Test that restarting a task while editing its path work as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restart_and_edit_task_path() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a task and wait for it to finish.
    assert_success(add_task(shared, "ls").await?);
    wait_for_task_condition(shared, 0, Task::is_done).await?;

    // Set the editor to a command which replaces the temporary file's content.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "echo '/tmp' > ${PUEUE_EDIT_PATH}/0/path ||");

    // Restart the command, edit its command and wait for it to start.
    run_client_command_with_env(shared, &["restart", "--in-place", "--edit", "0"], envs)?
        .success()?;
    wait_for_task_condition(shared, 0, Task::is_done).await?;

    // Make sure that the path has been updated.
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
    assert_success(add_task(shared, "ls").await.unwrap());
    wait_for_task_condition(shared, 0, Task::is_done)
        .await
        .unwrap();

    // Set the editor to a command which replaces the temporary file's content.
    let mut envs = HashMap::new();
    envs.insert(
        "EDITOR",
        "echo 'doesnotexist' > ${PUEUE_EDIT_PATH}/0/command && \
echo '/tmp' > ${PUEUE_EDIT_PATH}/0/path && \
echo 'label' > ${PUEUE_EDIT_PATH}/0/label && \
echo '5' > ${PUEUE_EDIT_PATH}/0/priority || ",
    );

    // Restart the command, edit its command and path and wait for it to start.
    // The task will fail afterwards, but it should still be edited.
    run_client_command_with_env(shared, &["restart", "--in-place", "--edit", "0"], envs)?
        .success()?;
    wait_for_task_condition(shared, 0, Task::is_done).await?;

    // Make sure that all properties have been updated.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.command, "doesnotexist");
    assert_eq!(task.path.to_string_lossy(), "/tmp");
    assert_eq!(task.label, Some("label".to_string()));
    assert_eq!(task.priority, 5);

    // Also the task should have been restarted and failed.
    assert_matches!(
        task.status,
        TaskStatus::Done {
            result: TaskResult::Failed(127),
            ..
        },
        "The task should have failed"
    );

    Ok(())
}

/// Test that restarting a task while editing its priority works as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restart_and_edit_task_priority() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a task and wait for it to finish.
    assert_success(add_task(shared, "ls").await?);
    wait_for_task_condition(shared, 0, Task::is_done).await?;

    // Set the editor to a command which replaces the temporary file's content.
    let mut envs = HashMap::new();
    envs.insert("EDITOR", "echo '99' > ${PUEUE_EDIT_PATH}/0/priority ||");

    // Restart the command, edit its priority and wait for it to start.
    run_client_command_with_env(shared, &["restart", "--in-place", "--edit", "0"], envs)?
        .success()?;
    wait_for_task_condition(shared, 0, Task::is_done).await?;

    // Make sure that the priority has been updated.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.priority, 99);

    Ok(())
}

/// Test that restarting a task **not** in place works as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn normal_restart_with_edit() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a task and wait for it to finish.
    assert_success(add_task(shared, "ls").await?);
    let original_task = wait_for_task_condition(shared, 0, Task::is_done).await?;

    // Set the editor to a command which replaces the temporary file's content.
    let mut envs = HashMap::new();
    envs.insert(
        "EDITOR",
        "echo 'sleep 60' > ${PUEUE_EDIT_PATH}/0/command ||",
    );

    // Restart the command, edit its command and wait for it to start.
    run_client_command_with_env(shared, &["restart", "--edit", "0"], envs)?.success()?;
    wait_for_task_condition(shared, 1, Task::is_running).await?;

    // Make sure that the command has been updated.
    let state = get_state(shared).await?;
    let task = state.tasks.get(&1).unwrap();
    assert_eq!(task.command, "sleep 60");
    assert_matches!(
        task.status,
        TaskStatus::Running { .. },
        "Task should be running"
    );

    // Since we created a copy, the new task should be created after the first one.
    assert!(
        original_task.created_at < task.created_at,
        "New task should have a newer created_at."
    );

    Ok(())
}

/// While editing, the original commands should be used instead of the substituted aliased command
/// strings.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restart_edit_with_alias() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create the alias file.
    let mut aliases = HashMap::new();
    aliases.insert("before".into(), "before aliased".into());
    aliases.insert("after".into(), "after aliased".into());
    create_test_alias_file(daemon.tempdir.path(), aliases)?;

    // Add a single task that instantly finishes.
    assert_success(add_and_start_task(shared, "before").await?);
    wait_for_task_condition(shared, 0, Task::is_done).await?;

    // Update the task's command by piping a string to the temporary file.
    // However, make sure that the old command is `before` and not the aliased command!
    let mut envs = HashMap::new();
    envs.insert(
        "EDITOR",
        r#"[[ "$(cat ${PUEUE_EDIT_PATH}/0/command)" == "before" ]] \
&& echo "after" > "${PUEUE_EDIT_PATH}/0/command" ||"#,
    );
    run_client_command_with_env(shared, &["restart", "--edit", "0"], envs)?.success()?;

    // Make sure that the command has been updated and the aliase worked.
    let state = get_state(shared).await?;
    let task = state
        .tasks
        .get(&1)
        .context("Expected task to be restarted")?;
    assert_eq!(task.original_command, "after");
    assert_eq!(task.command, "after aliased");

    // All other properties should be unchanged.
    assert_eq!(task.path, daemon.tempdir.path());
    assert_eq!(task.label, None);
    assert_eq!(task.priority, 0);

    Ok(())
}
