use pueue_lib::network::message::*;

use super::{handle_response, selection_from_params};
use crate::{client::client::Client, internal_prelude::*};

/// Start tasks, groups or the daemon.
///
/// When specific tasks are started, they're either resumed from a paused state or force-started in
/// case they're queued or stashed.
///
/// When groups are started, they start to be processed as usual.
pub async fn start(
    client: &mut Client,
    task_ids: Vec<usize>,
    group: Option<String>,
    all: bool,
) -> Result<()> {
    client
        .send_request(StartMessage {
            tasks: selection_from_params(all, group, task_ids),
        })
        .await?;

    let response = client.receive_response().await?;

    handle_response(&client.style, response)
}
