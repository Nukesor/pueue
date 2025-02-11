use pueue_lib::network::message::*;

use super::handle_response;
use crate::{client::client::Client, internal_prelude::*};

/// Initiate a daemon shutdown
pub async fn shutdown(client: &mut Client) -> Result<()> {
    client.send_request(Shutdown::Graceful).await?;

    let response = client.receive_response().await?;

    handle_response(&client.style, response)
}
