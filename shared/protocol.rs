use ::anyhow::Result;
use ::async_std::net::TcpStream;
use ::async_std::prelude::*;
use ::log::debug;
use ::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ::std::io::Cursor;

use crate::message::*;

/// Convenience wrapper around send_bytes
/// Deserialize a message and feed the bytes into send_bytes
pub async fn send_message(message: &Message, socket: &mut TcpStream) -> Result<()> {
    debug!("Sending message: {:?}", message);
    // Prepare command for transfer and determine message byte size
    let payload = serde_json::to_string(message)
        .expect("Failed to serialize message.")
        .into_bytes();

    send_bytes(payload, socket).await
}

/// Send a Vec of bytes. Before the actual bytes are send, the size of the message
/// is transmitted in an header of fixed size (u64).
pub async fn send_bytes(payload: Vec<u8>, socket: &mut TcpStream) -> Result<()> {
    let message_size = payload.len() as u64;

    let mut header = vec![];
    header.write_u64::<BigEndian>(message_size).unwrap();

    // Send the request size header first.
    // Afterwards send the request.
    socket.write_all(&header).await?;

    // Split the payload into 1.5kbyte chunks (MUT for TCP)
    let mut iter = payload.chunks(1500);
    while let Some(chunk) = iter.next() {
        socket.write_all(chunk).await?;
    }

    Ok(())
}

/// Receive a byte stream depending on a given header
/// This is the basic protocol beneath all pueue communication
pub async fn receive_bytes(socket: &mut TcpStream) -> Result<Vec<u8>> {
    // Receive the header with the overall message size
    let mut header = vec![0; 8];
    socket.read(&mut header).await?;
    let mut header = Cursor::new(header);
    let message_size = header.read_u64::<BigEndian>()? as usize;

    // Buffer for the whole payload
    let mut payload_bytes = Vec::new();

    // Receive chunks until we reached the expected message size
    while payload_bytes.len() < message_size {
        let mut remaining = message_size - payload_bytes.len();
        if remaining > 1500 {
            remaining = 1500;
        }
        let mut chunk_bytes = vec![0; remaining];
        socket.read(&mut chunk_bytes).await?;
        payload_bytes.append(&mut chunk_bytes);
    }

    Ok(payload_bytes)
}

/// Convenience wrapper that receives a message and converts it into a Message
pub async fn receive_message(socket: &mut TcpStream) -> Result<Message> {
    let payload_bytes = receive_bytes(socket).await?;

    // Deserialize the message
    let message = String::from_utf8(payload_bytes)?;
    debug!("Received message: {:?}", message);
    let message: Message = serde_json::from_str(&message)?;

    Ok(message)
}
