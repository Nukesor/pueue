use std::collections::HashMap;

use anyhow::{Context, Result};

use pueue_lib::network::message::*;
use pueue_lib::settings::*;

use super::*;

pub fn add_message(shared: &Shared, command: &str) -> AddMessage {
    AddMessage {
        command: command.into(),
        path: shared.pueue_directory().to_str().unwrap().to_string(),
        envs: HashMap::new(),
        start_immediately: false,
        stashed: false,
        group: PUEUE_DEFAULT_GROUP.into(),
        enqueue_at: None,
        dependencies: vec![],
        label: None,
        print_task_id: false,
    }
}

/// Helper to pause the whole daemon
pub async fn add_task(shared: &Shared, command: &str, start_immediately: bool) -> Result<Message> {
    let mut inner_message = add_message(shared, command);
    inner_message.start_immediately = start_immediately;
    let message = Message::Add(inner_message);

    send_message(shared, message)
        .await
        .context("Failed to to add task.")
}

/// Helper to pause the whole daemon
pub async fn add_task_to_group(shared: &Shared, command: &str, group: &str) -> Result<Message> {
    let message = Message::Add(AddMessage {
        command: command.into(),
        path: shared.pueue_directory().to_str().unwrap().to_string(),
        envs: HashMap::new(),
        start_immediately: false,
        stashed: false,
        group: group.to_owned(),
        enqueue_at: None,
        dependencies: vec![],
        label: None,
        print_task_id: false,
    });

    send_message(shared, message)
        .await
        .context("Failed to to add task to group.")
}

/// Mini wrapper around add_task, which always makes processes print their worker envs as well.
pub async fn add_env_task(shared: &Shared, command: &str) -> Result<Message> {
    let command = format!(
        "echo WORKER_ID: $PUEUE_WORKER_ID; echo GROUP: $PUEUE_GROUP; {}",
        command
    );
    fixtures::add_task(shared, &command, false).await
}

/// Just like [add_env_task], but task get's added to specific group.
pub async fn add_env_task_to_group(shared: &Shared, command: &str, group: &str) -> Result<Message> {
    let command = format!(
        "echo WORKER_ID: $PUEUE_WORKER_ID; echo GROUP: $PUEUE_GROUP; {}",
        command
    );
    fixtures::add_task_to_group(shared, &command, group).await
}

/// Create a new group with a specific amount of slots.
pub async fn add_group_with_slots(shared: &Shared, group_name: &str, slots: usize) -> Result<()> {
    let add_message = Message::Group(GroupMessage::Add(group_name.to_string(), Some(slots)));
    assert_success(send_message(shared, add_message.clone()).await?);
    wait_for_group(shared, group_name).await?;
    assert_success(send_message(shared, add_message.clone()).await?);

    Ok(())
}
