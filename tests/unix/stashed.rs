use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local};
use rstest::rstest;

use pueue_lib::network::message::*;
use pueue_lib::settings::Shared;
use pueue_lib::task::*;

use crate::helper::*;

/// Helper to pause the whole daemon
pub async fn add_stashed_task(
    shared: &Shared,
    command: &str,
    stashed: bool,
    enqueue_at: Option<DateTime<Local>>,
) -> Result<Message> {
    let message = Message::Add(AddMessage {
        command: command.into(),
        path: shared.pueue_directory().to_str().unwrap().to_string(),
        envs: HashMap::new(),
        start_immediately: false,
        stashed,
        group: "default".into(),
        enqueue_at,
        dependencies: vec![],
        label: None,
        print_task_id: false,
    });

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
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    let response = add_stashed_task(shared, "sleep 10", stashed, enqueue_at).await?;
    assert!(matches!(response, Message::Success(_)));

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
    wait_for_task_condition(shared, 0, |task| matches!(task.status, TaskStatus::Running)).await?;

    Ok(())
}

/// Delayed stashed tasks will be enqueued.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delayed_tasks() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // The task will be stashed and automatically enqueued after about 1 second.
    let response = add_stashed_task(
        shared,
        "sleep 10",
        true,
        Some(Local::now() + Duration::seconds(1)),
    )
    .await?;
    assert!(matches!(response, Message::Success(_)));

    // The task should be added in stashed state for about 1 second.
    wait_for_task_condition(shared, 0, |task| {
        matches!(task.status, TaskStatus::Stashed { .. })
    })
    .await?;

    // Make sure the task is started after being automatically enqueued.
    sleep_ms(800);
    wait_for_task_condition(shared, 0, |task| matches!(task.status, TaskStatus::Running)).await?;

    Ok(())
}
