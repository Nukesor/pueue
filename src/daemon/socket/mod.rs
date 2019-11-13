mod instructions;

use ::std::io::Cursor;
use ::std::sync::mpsc::Sender;
use ::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ::anyhow::Result;
use ::tokio::net::{TcpListener, TcpStream};
use ::tokio::prelude::*;

use crate::communication::message::*;
use crate::daemon::state::SharedState;
use crate::daemon::socket::instructions::handle_message;
use crate::settings::Settings;



/// Poll the unix listener and accept new incoming connections
/// Create a new future to handle the message and spawn it
pub async fn accept_incoming(_settings: Settings, sender: Sender<Message>, state: SharedState) -> Result<()> {
    let mut listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        // Poll if we have a new incoming connection.
        let (socket, _) = listener.accept().await?;
        let sender_clone = sender.clone();
        let state_clone  = state.clone();
        tokio::spawn(async move {
            let _result = handle_incoming(socket, sender_clone, state_clone).await;
        });
    }
}

/// Continuously poll the existing incoming futures.
/// In case we received an instruction, handle it and create a response future.
/// The response future is added to unix_responses and handled in a separate function.
pub async fn handle_incoming(mut socket: TcpStream, sender: Sender<Message>, state: SharedState) -> Result<()> {
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
    println!("{}", instruction);
    let message: Message = serde_json::from_str(&instruction)?;

    // Process the message
    let response = handle_message(message, sender, state)?;

    // Respond to the client
    send_message(socket, response).await?;

    Ok(())
}


/// Create the response future for this message.
async fn send_message(mut socket: TcpStream, message: Message) -> Result<()> {
    let payload = serde_json::to_string(&message)?.into_bytes();
    let byte_size = payload.len() as u64;

    let mut header = vec![];
    header.write_u64::<BigEndian>(byte_size).unwrap();

    socket.write_all(&header).await?;
    socket.write_all(&payload).await?;
    println!("Response sent");

    Ok(())
}


//pub fn handle_message(message: Message) -> Result<String> {
//    match message {
//        Message::Add(message_in) => add_task(&mut self.queue, message_in),
//        Message::Remove(message_in) => {
//            remove_task(&mut self.queue, &mut self.task_handler, message_in)
//        }
//        _ => Ok(Message::Failure(FailureMessage {
//            text: String::from("Unhandled message type."),
//        })),
//    }
//}
