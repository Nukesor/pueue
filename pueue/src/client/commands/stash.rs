use chrono::{DateTime, Local};
use pueue_lib::{client::Client, network::message::StashMessage};

use super::{handle_response, selection_from_params};
use crate::{client::style::OutputStyle, internal_prelude::*};

/// Tell the daemon to stash some tasks
pub async fn stash(
    client: &mut Client,
    style: &OutputStyle,
    task_ids: Vec<usize>,
    group: Option<String>,
    all: bool,
    delay_until: Option<DateTime<Local>>,
) -> Result<()> {
    let selection = selection_from_params(all, group, task_ids);
    client
        .send_request(StashMessage {
            tasks: selection,
            enqueue_at: delay_until,
        })
        .await?;

    let response = client.receive_response().await?;

    handle_response(style, response)
}
