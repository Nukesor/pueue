use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};
use async_std::task;
use clap::crate_version;
use crossbeam_channel::Sender;
use log::{debug, info, warn};

use pueue_lib::error::Error;
use pueue_lib::network::message::*;
use pueue_lib::network::protocol::*;
use pueue_lib::network::secret::read_shared_secret;
use pueue_lib::state::SharedState;

use crate::network::follow_log::handle_follow;
use crate::network::message_handler::handle_message;

/// Poll the listener and accept new incoming connections.
/// Create a new future to handle the message and spawn it.
pub async fn accept_incoming(sender: Sender<Message>, state: SharedState) -> Result<()> {
    // Get cloned shared settings to avoid holding a mutex over an await point.
    let shared_settings = {
        let state = state.lock().unwrap();
        state.settings.shared.clone()
    };
    let listener = get_listener(&shared_settings).await?;
    let secret = read_shared_secret(&shared_settings.shared_secret_path())?;

    loop {
        // Poll incoming connections.
        let stream = match listener.accept().await {
            Ok(stream) => stream,
            Err(err) => {
                warn!("Failed connecting to client: {:?}", err);
                continue;
            }
        };

        // Start a new task for the request
        let sender_clone = sender.clone();
        let state_clone = state.clone();
        let secret_clone = secret.clone();
        task::spawn(async move {
            let _result = handle_incoming(stream, sender_clone, state_clone, secret_clone).await;
        });
    }
}

/// Continuously poll the existing incoming futures.
/// In case we received an instruction, handle it and create a response future.
/// The response future is added to unix_responses and handled in a separate function.
async fn handle_incoming(
    mut stream: GenericStream,
    sender: Sender<Message>,
    state: SharedState,
    secret: Vec<u8>,
) -> Result<()> {
    // Receive the secret once and check, whether the client is allowed to connect
    let payload_bytes = receive_bytes(&mut stream).await?;

    // Didn't receive any bytes. The client disconnected.
    if payload_bytes.is_empty() {
        info!("Client went away");
        return Ok(());
    }

    let start = SystemTime::now();

    // Return immediately, if we got a wrong secret from the client.
    if payload_bytes != secret {
        warn!(
            "Received invalid secret: {}",
            String::from_utf8(payload_bytes)?
        );

        // Wait for 1 second before closing the socket, when getting a invalid secret.
        // This invalidates any timing attacks.
        let remaining_sleep_time = Duration::from_millis(1)
            - SystemTime::now()
                .duration_since(start)
                .context("Couldn't calculate duration. Did the system time change?")?;
        std::thread::sleep(remaining_sleep_time);
        bail!("Received invalid secret");
    }

    // Send a short `ok` byte to the client, so it knows that the secret has been accepted.
    // This is also the current version of the daemon, so the client can inform the user if the
    // daemon needs a restart in case a version difference exists.
    send_bytes(crate_version!().as_bytes(), &mut stream).await?;

    // Save the directory for convenience purposes and to prevent continuously
    // locking the state in the streaming loop.
    let pueue_directory = {
        let state = state.lock().unwrap();
        state.settings.shared.pueue_directory().clone()
    };

    loop {
        // Receive the actual instruction from the client
        let message_result = receive_message(&mut stream).await;

        if let Err(Error::EmptyPayload) = message_result {
            debug!("Client went away");
            return Ok(());
        }

        // In case of a deserialization error, respond the error to the client and return early.
        if let Err(Error::MessageDeserialization(err)) = message_result {
            send_message(
                create_failure_message(format!("Failed to deserialize message: {}", err)),
                &mut stream,
            )
            .await?;
            return Ok(());
        }

        let message = message_result?;
        debug!("Received instruction: {:?}", message);

        let response = if let Message::StreamRequest(message) = message {
            // The client requested the output of a task.
            // Since we allow streaming, this needs to be handled seperately.
            handle_follow(&pueue_directory, &mut stream, &state, message).await?
        } else {
            // Process a normal message.
            handle_message(message, &sender, &state)
        };

        // Respond to the client.
        send_message(response, &mut stream).await?;
    }
}
