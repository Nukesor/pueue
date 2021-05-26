//use anyhow::{anyhow, bail, Context, Result};
//
//use pueue_lib::network::message::*;
//use pueue_lib::settings::Shared;
//use pueue_lib::state::State;
//
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
