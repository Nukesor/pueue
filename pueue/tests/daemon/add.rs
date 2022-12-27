use anyhow::Result;

use chrono::Local;
use pueue_lib::network::message::TaskSelection;
use pueue_lib::task::*;

use crate::fixtures::*;
use crate::helper::*;

/// Test if adding a normal task works as intended.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_normal_add() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    let pre_addition_time = Local::now();

    // Add a task that instantly finishes
    assert_success(add_task(shared, "sleep 0.01", false).await?);

    // Wait until the task finished and get state
    let task = wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let post_addition_time = Local::now();

    // Make sure the task's created_at and enqueue_at times are viable.
    assert!(
        task.created_at > pre_addition_time && task.created_at < post_addition_time,
        "Make sure the created_at time is set correctly"
    );
    assert!(
        task.enqueued_at.unwrap() > pre_addition_time
            && task.enqueued_at.unwrap() < post_addition_time,
        "Make sure the enqueue_at time is set correctly"
    );

    // The task finished successfully
    assert_eq!(
        get_task_status(shared, 0).await?,
        TaskStatus::Done(TaskResult::Success)
    );

    Ok(())
}

/// Test if adding a task in stashed state work.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stashed_add() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Tell the daemon to add a task in stashed state.
    let mut message = create_add_message(shared, "sleep 60");
    message.stashed = true;
    assert_success(send_message(shared, message).await?);

    // Make sure the task is actually stashed.
    let task = wait_for_task_condition(shared, 0, |task| {
        matches!(task.status, TaskStatus::Stashed { .. })
    })
    .await?;

    assert!(
        task.enqueued_at.is_none(),
        "An unqueued task shouldn't have enqueue_at set."
    );

    Ok(())
}

/// Pause the default group and make sure that immediately spawning a task still works.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_add_with_immediate_start() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Pause the daemon and prevent tasks to be automatically spawned.
    pause_tasks(shared, TaskSelection::All).await?;

    // Tell the daemon to add a task that must be immediately started.
    assert_success(add_task(shared, "sleep 60", true).await?);

    // Make sure the task is actually being started.
    wait_for_task_condition(shared, 0, |task| task.is_running()).await?;

    Ok(())
}
