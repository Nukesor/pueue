use std::collections::HashMap;

use anyhow::{Context, Result};

use pueue_lib::network::message::*;
use pueue_lib::settings::*;

use super::*;

/// Helper to pause the whole daemon
pub async fn add_task(shared: &Shared, command: &str, start_immediately: bool) -> Result<Message> {
    let message = Message::Add(AddMessage {
        command: command.into(),
        path: shared.pueue_directory().to_str().unwrap().to_string(),
        envs: HashMap::new(),
        start_immediately,
        stashed: false,
        group: "default".into(),
        enqueue_at: None,
        dependencies: vec![],
        label: None,
        print_task_id: false,
    });

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
