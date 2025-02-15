use pueue_lib::{client::Client, network::message::*};

use super::handle_response;
use crate::{client::style::OutputStyle, internal_prelude::*};

/// Switch two queued or stashed tasks.
pub async fn switch(
    client: &mut Client,
    style: &OutputStyle,
    task_id_1: usize,
    task_id_2: usize,
) -> Result<()> {
    client
        .send_request(SwitchMessage {
            task_id_1,
            task_id_2,
        })
        .await?;

    let response = client.receive_response().await?;

    handle_response(style, response)
}
