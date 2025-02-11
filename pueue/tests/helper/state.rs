use pueue_lib::{network::message::*, settings::*, state::State};

use super::*;

/// Convenience function for getting the current state from the daemon.
pub async fn get_state(shared: &Shared) -> Result<Box<State>> {
    let response = send_request(shared, Request::Status).await?;
    match response {
        Response::Status(state) => Ok(state),
        _ => bail!("Didn't get status response in get_state"),
    }
}
