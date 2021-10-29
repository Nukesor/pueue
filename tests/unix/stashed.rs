use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local};
use rstest::rstest;

use pueue_lib::network::message::*;
use pueue_lib::settings::Shared;
use pueue_lib::task::*;

use crate::factories::*;
use crate::fixtures::*;
use crate::helper::*;

/// Helper to pause the whole daemon
pub async fn add_stashed_task(
    shared: &Shared,
    command: &str,
    stashed: bool,
    enqueue_at: Option<DateTime<Local>>,
) -> Result<Message> {
    let mut inner_message = add_message(shared, command);
    inner_message.stashed = stashed;
    inner_message.enqueue_at = enqueue_at;
    let message = Message::Add(inner_message);

    send_message(shared, message)
        .await
        .context("Failed to to add task message")
}

#[rstest]
#[case(true, None)]
#[case(true, Some(Local::now() + Duration::minutes(2)))]
#[case(false, Some(Local::now() + Duration::minutes(2)))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Tasks can be stashed and scheduled for being enqueued at a specific point in time.
///
/// Furthermore these stashed tasks can then be manually enqueued again.
async fn test_enqueued_tasks(
    #[case] stashed: bool,
    #[case] enqueue_at: Option<DateTime<Local>>,
) -> Result<()> {
    let daemon = daemon()?;
    let shared = &daemon.settings.shared;

    assert_success(add_stashed_task(shared, "sleep 10", stashed, enqueue_at).await?);

    // The task should be added in stashed state.
    wait_for_task_condition(shared, 0, |task| {
        matches!(task.status, TaskStatus::Stashed { .. })
    })
    .await?;

    // Assert the correct point in time has been set, in case `enqueue_at` is specific.
    if enqueue_at.is_some() {
        let status = get_task_status(shared, 0).await?;
        assert!(matches!(status, TaskStatus::Stashed { .. }));

        if let TaskStatus::Stashed { enqueue_at: inner } = status {
            assert_eq!(inner, enqueue_at);
        }
    }

    // Manually enqueue the task
    let enqueue_message = Message::Enqueue(EnqueueMessage {
        task_ids: vec![0],
        enqueue_at: None,
    });
    send_message(shared, enqueue_message)
        .await
        .context("Failed to to add task message")?;

    // Make sure the task is started after being enqueued
    wait_for_task_condition(shared, 0, |task| task.is_running()).await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Delayed stashed tasks will be enqueued.
async fn test_delayed_tasks() -> Result<()> {
    let daemon = daemon()?;
    let shared = &daemon.settings.shared;

    // The task will be stashed and automatically enqueued after about 1 second.
    let response = add_stashed_task(
        shared,
        "sleep 10",
        true,
        Some(Local::now() + Duration::seconds(1)),
    )
    .await?;
    assert_success(response);

    // The task should be added in stashed state for about 1 second.
    wait_for_task_condition(shared, 0, |task| {
        matches!(task.status, TaskStatus::Stashed { .. })
    })
    .await?;

    // Make sure the task is started after being automatically enqueued.
    sleep_ms(800);
    wait_for_task_condition(shared, 0, |task| task.is_running()).await?;

    Ok(())
}
