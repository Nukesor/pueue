use std::sync::mpsc::Sender;
use std::sync::Arc;

use anyhow::{bail, Result};
use async_std::task;
use async_tls::TlsAcceptor;
use log::{debug, info, warn};

use pueue::message::*;
use pueue::protocol::*;
use pueue::state::SharedState;
use pueue::tls::load_config;

use crate::cli::CliArguments;
use crate::instructions::handle_message;
use crate::streaming::handle_follow;

/// Poll the listener and accept new incoming connections.
/// Create a new future to handle the message and spawn it.
pub async fn accept_incoming(
    sender: Sender<Message>,
    state: SharedState,
    opt: CliArguments,
) -> Result<()> {
    let (unix_socket_path, tcp_info) = {
        let state = state.lock().unwrap();
        let shared = &state.settings.shared;

        // Return the unix socket path, if we're supposed to use it.
        if shared.use_unix_socket {
            (Some(shared.unix_socket_path.clone()), None)
        } else {
            // Otherwise use tcp sockets on a given port and host.
            // Commandline argument overwrites the configuration files values for port.
            // This also initializes the TLS acceptor.
            let port = if let Some(port) = opt.port.clone() {
                port
            } else {
                shared.port.clone()
            };

            let config = load_config(&state.settings)?;
            let acceptor = TlsAcceptor::from(Arc::new(config));
            (None, Some((port, acceptor)))
        }
    };

    let listener = get_listener(unix_socket_path, tcp_info.clone()).await?;

    loop {
        // Poll incoming connections.
        // We have to decide between Unix sockets and TCP sockets.
        // In case of a TCP connection, we have to add a TLS layer.
        let stream = if let Some((_, acceptor)) = tcp_info.clone() {
            let stream = listener.accept().await?;
            Box::new(acceptor.accept(stream).await?)
        } else {
            listener.accept().await?
        };

        //let socket = if let Some((_, ref acceptor)) = tcp_info {
        //    let stream = listener.accept().await?;
        //    Box::new(acceptor.accept(stream)?)
        //} else {
        //    listener.accept().await?
        //};

        // Start a new task for the request
        let sender_clone = sender.clone();
        let state_clone = state.clone();
        task::spawn(async move {
            let _result = handle_incoming(stream, sender_clone, state_clone).await;
        });
    }
}

/// Continuously poll the existing incoming futures.
/// In case we received an instruction, handle it and create a response future.
/// The response future is added to unix_responses and handled in a separate function.
async fn handle_incoming(
    mut socket: Socket,
    sender: Sender<Message>,
    state: SharedState,
) -> Result<()> {
    // Receive the secret once and check, whether the client is allowed to connect
    let payload_bytes = receive_bytes(&mut socket).await?;

    // Didn't receive any bytes. The client disconnected.
    if payload_bytes.is_empty() {
        info!("Client went away");
        return Ok(());
    }

    let secret = String::from_utf8(payload_bytes)?;

    // Return immediately, if we got a wrong secret from the client.
    {
        let state = state.lock().unwrap();
        if secret != state.settings.shared.secret {
            warn!("Received invalid secret: {}", secret);
            bail!("Received invalid secret");
        }
    }
    // Send a super short `ok` byte to the client, so it knows that the secret has been accepted.
    send_bytes(b"hello", &mut socket).await?;

    // Save the directory for convenience purposes and to prevent continuously
    // locking the state in the streaming loop.
    let pueue_directory = {
        let state = state.lock().unwrap();
        state.settings.shared.pueue_directory.clone()
    };

    loop {
        // Receive the actual instruction from the client
        let message = receive_message(&mut socket).await?;
        debug!("Received instruction: {:?}", message);

        let response = if let Message::StreamRequest(message) = message {
            // The client requested the output of a task.
            // Since we allow streaming, this needs to be handled seperately.
            handle_follow(&pueue_directory, &mut socket, &state, message).await?
        } else {
            // Process a normal message.
            handle_message(message, &sender, &state)
        };

        // Respond to the client.
        send_message(response, &mut socket).await?;
    }
}
