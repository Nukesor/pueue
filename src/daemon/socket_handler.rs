use ::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ::std::io::Cursor;
use ::anyhow::Result;
use ::tokio::net::{TcpListener, TcpStream};
use ::tokio::prelude::*;

use crate::communication::message::*;
use crate::settings::Settings;

/// Poll the unix listener and accept new incoming connections
/// Create a new future to handle the message and spawn it
pub async fn accept_incoming(settings: &Settings) -> Result<()> {
    let mut listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        // Poll if we have a new incoming connection.
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_incoming(socket).await;
        });
    }
}

/// Continuously poll the existing incoming futures.
/// In case we received an instruction, handle it and create a response future.
/// The response future is added to unix_responses and handled in a separate function.
pub async fn handle_incoming(mut socket: TcpStream) -> Result<()> {
    // Receive the header with the size and type of the message
    let mut header = vec![0; 8];
    socket.read(&mut header).await?;

    // Extract the instruction size from the header bytes
    let mut header = Cursor::new(header);
    let instruction_size = header.read_u64::<BigEndian>()? as usize;

    let mut instruction_bytes = vec![0; instruction_size];
    socket.read(&mut instruction_bytes).await?;

    let instruction = String::from_utf8(instruction_bytes)?;
    println!("{}", instruction);

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
