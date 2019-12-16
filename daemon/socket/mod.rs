mod instructions;
mod send;
mod stream;

use ::anyhow::Result;
use ::async_std::net::{TcpListener, TcpStream};
use ::async_std::prelude::*;
use ::async_std::task;
use ::byteorder::{BigEndian, ReadBytesExt};
use ::log::info;
use ::std::io::Cursor;
use ::std::sync::mpsc::Sender;

use crate::socket::instructions::handle_message;
use crate::socket::send::send_message;
use crate::socket::stream::handle_show;
use crate::cli::Opt;
use ::pueue::message::*;
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
    // Commandline argument overwrites the configuration files values for address
    let address = if let Some(address) = opt.address.clone() {
        address
    } else {
        settings.daemon.address.clone()
    };

    // Commandline argument overwrites the configuration files values for port
    let port = if let Some(port) = opt.port.clone() {
        port
    } else {
        settings.daemon.port.clone()
    };
    let address = format!("{}:{}", address, port);
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
pub async fn handle_incoming(
    mut socket: TcpStream,
    sender: Sender<Message>,
    state: SharedState,
    settings: Settings,
) -> Result<()> {
    loop {
        // Receive the header with the size and type of the message
        let mut header = vec![0; 8];
        socket.read(&mut header).await?;

        // Extract the instruction size from the header bytes
        let mut header = Cursor::new(header);
        let instruction_size = header.read_u64::<BigEndian>()? as usize;
        let mut instruction_bytes = vec![0; instruction_size];
        socket.read(&mut instruction_bytes).await?;

        // Receive the message and deserialize it
        let instruction = String::from_utf8(instruction_bytes)?;
        let message: Message = serde_json::from_str(&instruction)?;
        info!("Received instruction: {}", instruction);

        let response = if let Message::StreamRequest(message) = message {
            // The client requested the output of a task
            // Since we allow streaming, this needs to be handled seperately
            handle_show(&settings, &mut socket, message).await?
        } else {
            // Process a normal message
            handle_message(message, &sender, &state)
        };

        // Respond to the client
        send_message(&mut socket, response).await?;
    }
}
