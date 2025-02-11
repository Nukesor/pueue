use pueue_lib::network::message::*;

use super::{handle_response, selection_from_params};
use crate::{client::client::Client, internal_prelude::*};

/// Pause some running tasks, a group or all groups.
///
/// When pausing groups or the daemon, the `wait` flag can be used to let running tasks finish.
pub async fn pause(
    client: &mut Client,
    task_ids: Vec<usize>,
    group: Option<String>,
    all: bool,
    wait: bool,
) -> Result<()> {
    client
        .send_request(PauseMessage {
            tasks: selection_from_params(all, group, task_ids),
            wait,
        })
        .await?;

    let response = client.receive_response().await?;

    handle_response(&client.style, response)
}
