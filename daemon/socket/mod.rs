mod instructions;

use ::anyhow::Result;
use ::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ::log::info;
use ::std::io::Cursor;
use ::std::sync::mpsc::Sender;
use ::async_std::net::{TcpListener, TcpStream};
use ::async_std::task;
use ::async_std::prelude::*;

use crate::socket::instructions::handle_message;
use ::pueue::message::*;
use ::pueue::settings::Settings;
use ::pueue::state::SharedState;

/// Poll the unix listener and accept new incoming connections
/// Create a new future to handle the message and spawn it
pub async fn accept_incoming(
    settings: Settings,
    sender: Sender<Message>,
    state: SharedState,
) -> Result<()> {
    let address = format!(
        "{}:{}",
        settings.client.daemon_address, settings.client.daemon_port
    );
    let listener = TcpListener::bind(address).await?;

    loop {
        // Poll if we have a new incoming connection.
        let (socket, _) = listener.accept().await?;
        let sender_clone = sender.clone();
        let state_clone = state.clone();
        task::spawn(async move {
            let _result = handle_incoming(socket, sender_clone, state_clone).await;
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

        // Process the message
        let response = handle_message(message, &sender, &state);

        // Respond to the client
        send_message(&mut socket, response).await?;
    }
}

/// Create the response future for this message.
async fn send_message(socket: &mut TcpStream, message: Message) -> Result<()> {
    let payload = serde_json::to_string(&message)?.into_bytes();
    let byte_size = payload.len() as u64;

    let mut header = vec![];
    header.write_u64::<BigEndian>(byte_size).unwrap();

    socket.write_all(&header).await?;
    socket.write_all(&payload).await?;

    Ok(())
}
