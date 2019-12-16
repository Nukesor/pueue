use ::anyhow::Result;
use ::async_std::net::TcpStream;
use ::async_std::prelude::*;
use ::std::io::Cursor;
use ::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::message::*;


/// Convenience wrapper around send_bytes
/// Deserialize a message and feed the bytes into send_bytes
pub async fn send_message(message: &Message, socket: &mut TcpStream) -> Result<()> {
    // Prepare command for transfer and determine message byte size
    let payload = serde_json::to_string(message)
        .expect("Failed to serialize message.")
        .into_bytes();

    send_bytes(payload, socket).await
}


/// Send a Vec of bytes. Before the actual bytes are send, the size of the message
/// is transmitted in an header of fixed size (u64).
pub async fn send_bytes(payload: Vec<u8>, socket: &mut TcpStream) -> Result<()> {
    let byte_size = payload.len() as u64;

    let mut header = vec![];
    header.write_u64::<BigEndian>(byte_size).unwrap();

    // Send the request size header first.
    // Afterwards send the request.
    socket.write_all(&header).await?;
    socket.write_all(&payload).await?;

    Ok(())
}


/// Receive a byte stream depending on a given header
/// This is the basic protocol beneath all pueue communication
pub async fn receive_bytes(socket: &mut TcpStream) -> Result<Vec<u8>> {
    // Receive the header with the size
    let mut header = vec![0; 8];
    socket.read(&mut header).await?;
    let mut header = Cursor::new(header);
    let message_size = header.read_u64::<BigEndian>()? as usize;

    // Receive the payload
    let mut payload_bytes = vec![0; message_size];
    socket.read(&mut payload_bytes).await?;
    Ok(payload_bytes)
}

/// Convenience wrapper that receives a message and converts it into a Message
pub async fn receive_message(socket: &mut TcpStream) -> Result<Message> {
    let payload_bytes = receive_bytes(socket).await?;

    // Deserialize the message
    let message = String::from_utf8(payload_bytes)?;
    let message: Message = serde_json::from_str(&message)?;

    Ok(message)
}
