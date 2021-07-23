use anyhow::{Context, Result};

use pueue_lib::network::message::*;
use pueue_lib::settings::*;

use super::*;

/// Helper to either continue the daemon or start specific tasks
pub async fn start_tasks(shared: &Shared, tasks: TaskSelection) -> Result<Message> {
    let message = Message::Start(StartMessage {
        tasks,
        children: false,
    });

    send_message(shared, message)
        .await
        .context("Failed to send Start tasks message")
}

/// Helper to pause the default group of the daemon
pub async fn pause_tasks(shared: &Shared, tasks: TaskSelection) -> Result<Message> {
    let message = Message::Pause(PauseMessage {
        tasks,
        wait: false,
        children: false,
    });

    send_message(shared, message)
        .await
        .context("Failed to send Pause message")
}

/// Helper to pause the whole daemon
pub async fn shutdown_daemon(shared: &Shared) -> Result<Message> {
    let message = Message::DaemonShutdown(Shutdown::Graceful);

    send_message(shared, message)
        .await
        .context("Failed to send Shutdown message")
}
