use ::anyhow::Result;
use ::byteorder::{BigEndian, WriteBytesExt};
use ::async_std::net::TcpStream;
use ::async_std::prelude::*;

use ::pueue::message::*;

/// Create the response future for this message.
pub async fn send_message(socket: &mut TcpStream, message: Message) -> Result<()> {
    let payload = serde_json::to_string(&message)?.into_bytes();
    let byte_size = payload.len() as u64;

    let mut header = vec![];
    header.write_u64::<BigEndian>(byte_size).unwrap();

    socket.write_all(&header).await?;
    socket.write_all(&payload).await?;

    Ok(())
}
