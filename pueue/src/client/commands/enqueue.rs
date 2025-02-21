use chrono::{DateTime, Local};
use pueue_lib::{client::Client, network::message::*};

use super::{handle_response, selection_from_params};
use crate::{client::style::OutputStyle, internal_prelude::*};

/// Enqueue a stashed task or schedule it to be enqueued at a specific point in time.
pub async fn enqueue(
    client: &mut Client,
    style: &OutputStyle,
    task_ids: Vec<usize>,
    group: Option<String>,
    all: bool,
    delay_until: Option<DateTime<Local>>,
) -> Result<()> {
    client
        .send_request(EnqueueRequest {
            tasks: selection_from_params(all, group, task_ids),
            enqueue_at: delay_until,
        })
        .await?;

    let response = client.receive_response().await?;

    handle_response(style, response)
}
