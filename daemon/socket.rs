use ::anyhow::{bail, Result};
use ::async_std::net::{TcpListener, TcpStream};
use ::async_std::task;
use ::log::{info, warn};
use ::std::sync::mpsc::Sender;

use crate::cli::Opt;
use crate::instructions::handle_message;
use crate::streaming::handle_show;
use ::pueue::message::*;
use ::pueue::protocol::*;
use ::pueue::settings::Settings;
use ::pueue::state::SharedState;

/// Poll the unix listener and accept new incoming connections
/// Create a new future to handle the message and spawn it
pub async fn accept_incoming(
    settings: Settings,
    sender: Sender<Message>,
    state: SharedState,
    opt: Opt,
) -> Result<()> {
    //    // Commandline argument overwrites the configuration files values for address
    //    let address = if let Some(address) = opt.address.clone() {
    //        address
    //    } else {
    //        settings.daemon.address.clone()
    //    };

    // Commandline argument overwrites the configuration files values for port
    let port = if let Some(port) = opt.port.clone() {
        port
    } else {
        settings.daemon.port.clone()
    };
    //    let address = format!("{}:{}", address, port);
    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(address).await?;

    loop {
        // Poll if we have a new incoming connection.
        let (socket, _) = listener.accept().await?;
        let sender_clone = sender.clone();
        let state_clone = state.clone();
        let settings_clone = settings.clone();
        task::spawn(async move {
            let _result = handle_incoming(socket, sender_clone, state_clone, settings_clone).await;
        });
    }
}

/// Continuously poll the existing incoming futures.
/// In case we received an instruction, handle it and create a response future.
/// The response future is added to unix_responses and handled in a separate function.
async fn handle_incoming(
    mut socket: TcpStream,
    sender: Sender<Message>,
    state: SharedState,
    settings: Settings,
) -> Result<()> {
    // Receive the secret once and check, whether the client is allowed to connect
    let payload_bytes = receive_bytes(&mut socket).await?;

    // Didn't receive any bytes. The client disconnected
    if payload_bytes.len() == 0 {
        info!("Client went away");
        return Ok(());
    }

    let secret = String::from_utf8(payload_bytes)?;
    if secret != settings.daemon.secret {
        warn!("Received invalid secret: {}", secret);
        bail!("Received invalid secret");
    }

    loop {
        // Receive the actual instruction from the client
        let message = receive_message(&mut socket).await?;
        info!("Received instruction: {:?}", message);

        let response = if let Message::StreamRequest(message) = message {
            // The client requested the output of a task
            // Since we allow streaming, this needs to be handled seperately
            handle_show(&settings, &mut socket, message).await?
        } else if let Message::DaemonShutdown = message {
            // Simply shut down the daemon right after sending a success response
            let response = create_success_message("Daemon is shutting down");
            send_message(response, &mut socket).await?;
            std::process::exit(0);
        } else {
            // Process a normal message
            handle_message(message, &sender, &state)
        };

        // Respond to the client
        send_message(response, &mut socket).await?;
    }
}
