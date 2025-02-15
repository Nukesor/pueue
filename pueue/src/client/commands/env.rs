use pueue_lib::{client::Client, network::message::EnvMessage};

use super::handle_response;
use crate::{
    client::{cli::EnvCommand, style::OutputStyle},
    internal_prelude::*,
};

/// Set or unset an environment variable on a task.
pub async fn env(client: &mut Client, style: &OutputStyle, cmd: EnvCommand) -> Result<()> {
    let request = match cmd {
        EnvCommand::Set {
            task_id,
            key,
            value,
        } => EnvMessage::Set {
            task_id,
            key,
            value,
        },
        EnvCommand::Unset { task_id, key } => EnvMessage::Unset { task_id, key },
    };

    client.send_request(request).await?;

    let response = client.receive_response().await?;

    handle_response(style, response)
}
