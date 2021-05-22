use anyhow::{anyhow, bail, Context, Result};

use pueue_lib::network::message::*;


use pueue_lib::settings::Shared;
use pueue_lib::state::State;

use super::network::*;

pub async fn get_state(shared: &Shared) -> Result<Box<State>> {
    let response = send_message(shared, Message::Status).await?;
    match response {
        Message::StatusResponse(state) => Ok(state),
        _ => bail!("Didn't get status response in get_state"),
    }
}

//pub async fn pause_daemon(shared: &Shared) -> Message {
//    let message = Message::Pause(PauseMessage {
//        task_ids: vec![],
//        group: "default".into(),
//        wait: false,
//        all: true,
//        children: false,
//    });
//
//    send_message(shared, message).await
//}

pub async fn shutdown(shared: &Shared) -> Result<()> {
    send_message(shared, Message::DaemonShutdown).await?;

    Ok(())
}
