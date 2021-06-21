use anyhow::{bail, Context, Result};

use pueue_lib::network::message::*;
use pueue_lib::settings::*;
use pueue_lib::state::State;

use super::*;

/// Helper to continue the daemon and all paused tasks
pub async fn start_tasks(shared: &Shared, task_ids: Vec<usize>) -> Result<Message> {
    let message = Message::Start(StartMessage {
        task_ids,
        group: "default".into(),
        all: false,
        children: false,
    });

    send_message(shared, message)
        .await
        .context("Failed to send Start tasks message")
}

/// Helper to continue the daemon and all paused tasks
pub async fn continue_daemon(shared: &Shared) -> Result<Message> {
    let message = Message::Start(StartMessage {
        task_ids: vec![],
        group: "default".into(),
        all: true,
        children: false,
    });

    send_message(shared, message)
        .await
        .context("Failed to send continue message")
}

/// Helper to pause the whole daemon
pub async fn pause_daemon(shared: &Shared) -> Result<Message> {
    let message = Message::Pause(PauseMessage {
        task_ids: vec![],
        group: "default".into(),
        wait: false,
        all: true,
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

pub async fn get_state(shared: &Shared) -> Result<Box<State>> {
    let response = send_message(shared, Message::Status).await?;
    match response {
        Message::StatusResponse(state) => Ok(state),
        _ => bail!("Didn't get status response in get_state"),
    }
}
