use pueue_lib::Task;

use crate::{client::helper::*, internal_prelude::*};

/// Test that removing all finished tasks with --all works.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remove_all() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create two tasks and wait for them to finish.
    assert_success(add_task(shared, "ls").await?);
    assert_success(add_task(shared, "echo test").await?);
    wait_for_task_condition(shared, 0, Task::is_done).await?;
    wait_for_task_condition(shared, 1, Task::is_done).await?;

    // Remove all finished tasks.
    run_client_command(shared, &["remove", "--all"])?.success()?;

    // Both tasks should be removed.
    let state = get_state(shared).await?;
    assert_eq!(state.tasks.len(), 0, "All tasks should be removed");

    Ok(())
}

/// Test that removing tasks in a specific group with --group works.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remove_group() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create tasks in different groups.
    let mut message = create_add_message(shared, "ls");
    message.group = "test_2".to_string();
    assert_success(send_request(shared, message).await?);

    assert_success(add_task(shared, "echo default").await?);

    wait_for_task_condition(shared, 0, Task::is_done).await?;
    wait_for_task_condition(shared, 1, Task::is_done).await?;

    // Remove only tasks in test_2 group.
    run_client_command(shared, &["remove", "--group", "test_2"])?.success()?;

    // Only task from test_2 should be removed.
    let state = get_state(shared).await?;
    assert_eq!(state.tasks.len(), 1, "Should have 1 task remaining");
    assert!(state.tasks.contains_key(&1), "Default group task should remain");
    assert!(!state.tasks.contains_key(&0), "test_2 group task should be removed");

    Ok(())
}

/// Test that --all and --group are mutually exclusive.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remove_all_and_group_conflict() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Try to use both --all and --group, should fail.
    let output = run_client_command(shared, &["remove", "--all", "--group", "test_2"])?;
    assert!(
        !output.status.success(),
        "Command should fail when using both --all and --group"
    );

    Ok(())
}
