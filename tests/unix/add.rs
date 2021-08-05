use anyhow::Result;

use pueue_lib::network::message::{Message, TaskSelection};
use pueue_lib::task::*;

use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Test if adding a normal task works as intended.
async fn test_normal_add() -> Result<()> {
    let (settings, _tempdir, _pid) = threaded_setup()?;
    let shared = &settings.shared;

    // Add a task that instantly finishes
    assert_success(fixtures::add_task(shared, "sleep 0.01", false).await?);

    // Wait until the task finished and get state
    wait_for_task_condition(&shared, 0, |task| task.is_done()).await?;

    // The task finished succesfully
    assert_eq!(
        get_task_status(shared, 0).await?,
        TaskStatus::Done(TaskResult::Success)
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Test if adding a task in stashed state work.
async fn test_stashed_add() -> Result<()> {
    let (settings, _tempdir, _pid) = threaded_setup()?;
    let shared = &settings.shared;

    // Tell the daemon to add a task in stashed state.
    let mut inner_message = fixtures::add_message(shared, "sleep 60");
    inner_message.stashed = true;
    let message = Message::Add(inner_message);
    assert_success(send_message(shared, message).await?);

    // Make sure the task is actually stashed.
    wait_for_task_condition(&shared, 0, |task| {
        matches!(task.status, TaskStatus::Stashed { .. })
    })
    .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Pause the default group and make sure that immediately spawning a task still works.
async fn test_add_with_immediate_start() -> Result<()> {
    let (settings, _tempdir, _pid) = threaded_setup()?;
    let shared = &settings.shared;

    // Pause the daemon and prevent tasks to be automatically spawned.
    pause_tasks(shared, TaskSelection::All).await?;

    // Tell the daemon to add a task that must be immediately started.
    assert_success(fixtures::add_task(shared, "sleep 60", true).await?);

    // Make sure the task is actually being started.
    wait_for_task_condition(&shared, 0, |task| task.is_running()).await?;

    Ok(())
}
