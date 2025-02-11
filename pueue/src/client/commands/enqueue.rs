use chrono::{DateTime, Local};
use pueue_lib::network::message::*;

use super::{handle_response, selection_from_params};
use crate::{client::client::Client, internal_prelude::*};

/// Enqueue a stashed task or schedule it to be enqueued at a specific point in time.
pub async fn enqueue(
    client: &mut Client,
    task_ids: Vec<usize>,
    group: Option<String>,
    all: bool,
    delay_until: Option<DateTime<Local>>,
) -> Result<()> {
    client
        .send_request(EnqueueMessage {
            tasks: selection_from_params(all, group, task_ids),
            enqueue_at: delay_until,
        })
        .await?;

    let response = client.receive_response().await?;

    handle_response(&client.style, response)
}
