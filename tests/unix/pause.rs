use anyhow::{Context, Result};
use pueue_lib::network::message::*;
use pueue_lib::state::GroupStatus;
use pueue_lib::task::*;

use crate::fixtures::*;
use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Make sure that no tasks will be started in a paused queue
async fn test_pause_daemon() -> Result<()> {
    let daemon = daemon()?;
    let shared = &daemon.settings.shared;

    // This pauses the daemon
    pause_tasks(shared, TaskSelection::All).await?;
    // Make sure the default group get's paused
    wait_for_group_status(shared, PUEUE_DEFAULT_GROUP, GroupStatus::Paused).await?;

    // Add a task and give the taskmanager time to theoretically start the process
    add_task(shared, "ls", false).await?;
    sleep_ms(500);

    // Make sure it's not started
    assert_eq!(get_task_status(shared, 0).await?, TaskStatus::Queued);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Make sure that running tasks will be properly paused
async fn test_pause_running_task() -> Result<()> {
    let daemon = daemon()?;
    let shared = &daemon.settings.shared;

    // Start a long running task and make sure it's started
    add_task(shared, "sleep 60", false).await?;
    wait_for_task_condition(shared, 0, |task| task.is_running()).await?;

    // This pauses the daemon
    pause_tasks(shared, TaskSelection::All).await?;

    // Make sure the task as well as the default group get paused
    wait_for_task_condition(shared, 0, |task| matches!(task.status, TaskStatus::Paused)).await?;
    let state = get_state(shared).await?;
    assert_eq!(
        state.groups.get(PUEUE_DEFAULT_GROUP).unwrap(),
        &GroupStatus::Paused
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// A queue can get paused, while the tasks may finish on their own.
async fn test_pause_with_wait() -> Result<()> {
    let daemon = daemon()?;
    let shared = &daemon.settings.shared;

    // Start a long running task and make sure it's started
    add_task(shared, "sleep 60", false).await?;
    wait_for_task_condition(shared, 0, |task| task.is_running()).await?;

    // Pauses the default queue while waiting for tasks
    let message = Message::Pause(PauseMessage {
        tasks: TaskSelection::Group(PUEUE_DEFAULT_GROUP.into()),
        wait: true,
        children: false,
    });
    send_message(shared, message)
        .await
        .context("Failed to send message")?;

    // Make sure the default group gets paused, but the task is still running
    wait_for_group_status(shared, PUEUE_DEFAULT_GROUP, GroupStatus::Paused).await?;
    let state = get_state(shared).await?;
    assert_eq!(state.tasks.get(&0).unwrap().status, TaskStatus::Running);

    Ok(())
}
