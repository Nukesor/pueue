use pueue_lib::network::message::*;

use super::{handle_response, handle_user_confirmation, selection_from_params};
use crate::{client::client::Client, internal_prelude::*};

/// Kill some running or paused task.
///
/// Can also be used to send a specific [`Signal`].
pub async fn kill(
    client: &mut Client,
    task_ids: Vec<usize>,
    group: Option<String>,
    all: bool,
    signal: Option<Signal>,
) -> Result<()> {
    if client.settings.client.show_confirmation_questions {
        handle_user_confirmation("kill", &task_ids)?;
    }

    client
        .send_request(KillMessage {
            tasks: selection_from_params(all, group, task_ids),
            signal,
        })
        .await?;

    let response = client.receive_response().await?;

    handle_response(&client.style, response)
}
