use anyhow::{anyhow, bail, Context, Result};

use pueue_lib::network::message::*;
use pueue_lib::network::protocol::{
    get_client_stream, receive_bytes, receive_message, send_bytes,
    send_message as internal_send_message, GenericStream,
};
use pueue_lib::network::secret::read_shared_secret;
use pueue_lib::settings::Shared;
use pueue_lib::state::State;

pub async fn send_message(shared: &Shared, message: Message) -> Result<Message> {
    let mut stream = get_authenticated_client(shared).await?;

    // Check if we can receive the response from the daemon
    internal_send_message(message, &mut stream)
        .await
        .map_err(|err| anyhow!("Failed to send message: {}", err))?;

    // Check if we can receive the response from the daemon
    receive_message(&mut stream)
        .await
        .map_err(|err| anyhow!("Failed to receive message: {}", err))
}

pub async fn get_authenticated_client(shared: &Shared) -> Result<GenericStream> {
    // Connect to daemon and get stream used for communication.
    let mut stream = match get_client_stream(shared).await {
        Ok(stream) => stream,
        Err(err) => {
            panic!("Couldn't get client stream: {}", err);
        }
    };

    // Next we do a handshake with the daemon
    // 1. Client sends the secret to the daemon.
    // 2. If successful, the daemon responds with their version.
    let secret =
        read_shared_secret(&shared.shared_secret_path()).context("Couldn't read shared secret.")?;
    send_bytes(&secret, &mut stream)
        .await
        .context("Failed to send bytes.")?;
    let version_bytes = receive_bytes(&mut stream)
        .await
        .context("Failed sending secret during handshake with daemon.")?;

    if version_bytes.is_empty() {
        bail!("Daemon went away after sending secret. Did you use the correct secret?")
    }

    Ok(stream)
}

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
