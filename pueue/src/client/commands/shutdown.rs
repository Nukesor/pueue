use pueue_lib::{client::Client, network::message::*};

use super::handle_response;
use crate::{client::style::OutputStyle, internal_prelude::*};

/// Initiate a daemon shutdown
pub async fn shutdown(client: &mut Client, style: &OutputStyle) -> Result<()> {
    client.send_request(ShutdownRequest::Graceful).await?;

    let response = client.receive_response().await?;

    handle_response(style, response)
}
