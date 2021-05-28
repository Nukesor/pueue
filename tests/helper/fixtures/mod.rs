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
        .context("Failed to to add task message")
}
