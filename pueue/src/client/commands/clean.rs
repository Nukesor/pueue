use pueue_lib::{client::Client, network::message::*};

use super::handle_response;
use crate::{client::style::OutputStyle, internal_prelude::*};

/// Tell the daemon to clear finished tasks for a specific group or the whole daemon.
///
/// The `successful_only` determines whether finished tasks should be removed or not.
pub async fn clean(
    client: &mut Client,
    style: &OutputStyle,
    group: Option<String>,
    successful_only: bool,
) -> Result<()> {
    client
        .send_request(CleanMessage {
            successful_only,
            group,
        })
        .await?;

    let response = client.receive_response().await?;

    handle_response(style, response)
}
