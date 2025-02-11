use pueue_lib::network::message::*;

use super::handle_response;
use crate::{client::client::Client, internal_prelude::*};

/// Send some input to a running task.
pub async fn send(client: &mut Client, task_id: usize, input: String) -> Result<()> {
    client.send_request(SendMessage { task_id, input }).await?;

    let response = client.receive_response().await?;

    handle_response(&client.style, response)
}
