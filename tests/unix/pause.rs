use anyhow::{Context, Result};
use pueue_lib::network::message::*;
use pueue_lib::state::GroupStatus;
use pueue_lib::task::*;

use crate::helper::fixtures::add_task;
use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Make sure that no tasks will be started in a paused queue
async fn test_pause_daemon() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // This pauses the daemon
    pause_tasks(shared, TaskSelection::All).await?;
    // Make sure the default group get's paused
    wait_for_group_status(shared, "default", GroupStatus::Paused).await?;

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
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // Start a long running task and make sure it's started
    add_task(shared, "sleep 60", false).await?;
    wait_for_task_condition(shared, 0, |task| matches!(task.status, TaskStatus::Running)).await?;

    // This pauses the daemon
    pause_tasks(shared, TaskSelection::All).await?;

    // Make sure the task as well as the default group get paused
    wait_for_task_condition(shared, 0, |task| matches!(task.status, TaskStatus::Paused)).await?;
    let state = get_state(shared).await?;
    assert_eq!(state.groups.get("default").unwrap(), &GroupStatus::Paused);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// A queue can get paused, while the tasks may finish on their own.
async fn test_pause_with_wait() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // Start a long running task and make sure it's started
    add_task(shared, "sleep 60", false).await?;
    wait_for_task_condition(shared, 0, |task| matches!(task.status, TaskStatus::Running)).await?;

    // Pauses the default queue while waiting for tasks
    let message = Message::Pause(PauseMessage {
        tasks: TaskSelection::Group("default".into()),
        wait: true,
        children: false,
    });
    send_message(shared, message)
        .await
        .context("Failed to send message")?;

    // Make sure the default group gets paused, but the task is still running
    wait_for_group_status(shared, "default", GroupStatus::Paused).await?;
    let state = get_state(shared).await?;
    assert_eq!(state.tasks.get(&0).unwrap().status, TaskStatus::Running);

    Ok(())
}
