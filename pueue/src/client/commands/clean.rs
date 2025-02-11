use pueue_lib::network::message::*;

use super::handle_response;
use crate::{client::client::Client, internal_prelude::*};

/// Tell the daemon to clear finished tasks for a specific group or the whole daemon.
///
/// The `successful_only` determines whether finished tasks should be removed or not.
pub async fn clean(
    client: &mut Client,
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

    handle_response(&client.style, response)
}
