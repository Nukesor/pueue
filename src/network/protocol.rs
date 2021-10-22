use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use log::debug;
use serde_cbor::de::from_slice;
use serde_cbor::ser::to_vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::Error;
use crate::network::message::*;

// Reexport all stream/socket related stuff for convenience purposes
pub use super::platform::socket::Stream;
pub use super::platform::socket::*;

/// Convenience wrapper around send_bytes.
/// Deserialize a message and feed the bytes into send_bytes.
pub async fn send_message(message: Message, stream: &mut GenericStream) -> Result<(), Error> {
    debug!("Sending message: {:?}", message);
    // Prepare command for transfer and determine message byte size
    let payload = to_vec(&message).map_err(|err| Error::MessageDeserialization(err.to_string()))?;

    send_bytes(&payload, stream).await
}

/// Send a Vec of bytes. Before the actual bytes are send, the size of the message
/// is transmitted in an header of fixed size (u64).
pub async fn send_bytes(payload: &[u8], stream: &mut GenericStream) -> Result<(), Error> {
    let message_size = payload.len() as u64;

    let mut header = vec![];
    WriteBytesExt::write_u64::<BigEndian>(&mut header, message_size).unwrap();

    // Send the request size header first.
    // Afterwards send the request.
    stream.write_all(&header).await?;

    // Split the payload into 1.4Kbyte chunks
    // 1.5Kbyte is the MUT for TCP, but some carrier have a little less, such as Wireguard.
    for chunk in payload.chunks(1400) {
        stream.write_all(chunk).await?;
    }

    Ok(())
}

/// Receive a byte stream. \
/// This is the basic protocol beneath all pueue communication. \
///
/// 1. The client sends a u64, which specifies the length of the payload.
/// 2. Receive chunks of 1400 bytes until we finished all expected bytes
pub async fn receive_bytes(stream: &mut GenericStream) -> Result<Vec<u8>, Error> {
    // Receive the header with the overall message size
    let mut header = vec![0; 8];
    stream.read_exact(&mut header).await?;
    let mut header = Cursor::new(header);
    let message_size = ReadBytesExt::read_u64::<BigEndian>(&mut header)? as usize;

    // Buffer for the whole payload
    let mut payload_bytes = Vec::with_capacity(message_size);

    // Create a static buffer with our packet size.
    let mut chunk_buffer: [u8; 1400] = [0; 1400];

    // Receive chunks until we reached the expected message size
    while payload_bytes.len() < message_size {
        // Read data and get the amount of received bytes
        let received_bytes = stream.read(&mut chunk_buffer).await?;

        if received_bytes == 0 {
            return Err(Error::Connection(
                "Connection went away while receiving payload.".into(),
            ));
        }

        // Extend the total payload bytes by the part of the buffer that has been filled
        // during this iteration.
        payload_bytes.extend_from_slice(&chunk_buffer[0..received_bytes]);
    }

    Ok(payload_bytes)
}

/// Convenience wrapper that receives a message and converts it into a Message.
pub async fn receive_message(stream: &mut GenericStream) -> Result<Message, Error> {
    let payload_bytes = receive_bytes(stream).await?;
    debug!("Received {} bytes", payload_bytes.len());
    if payload_bytes.is_empty() {
        return Err(Error::EmptyPayload);
    }

    // Deserialize the message.
    let message: Message =
        from_slice(&payload_bytes).map_err(|err| Error::MessageDeserialization(err.to_string()))?;
    debug!("Received message: {:?}", message);

    Ok(message)
}

#[cfg(test)]
mod test {
    use super::*;

    use async_trait::async_trait;
    use pretty_assertions::assert_eq;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::task;

    use crate::network::platform::socket::Stream as PueueStream;

    // Implement generic Listener/Stream traits, so we can test stuff on normal TCP
    #[async_trait]
    impl Listener for TcpListener {
        async fn accept<'a>(&'a self) -> Result<GenericStream, Error> {
            let (stream, _) = self.accept().await?;
            Ok(Box::new(stream))
        }
    }
    impl PueueStream for TcpStream {}

    #[tokio::test]
    async fn test_single_huge_payload() -> Result<(), Error> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        // The message that should be sent
        let payload = "a".repeat(100_000);
        let message = create_success_message(payload);
        let original_bytes = to_vec(&message).expect("Failed to serialize message.");

        let listener: GenericListener = Box::new(listener);

        // Spawn a sub thread that:
        // 1. Accepts a new connection
        // 2. Reads a message
        // 3. Sends the same message back
        task::spawn(async move {
            let mut stream = listener.accept().await.unwrap();
            let message_bytes = receive_bytes(&mut stream).await.unwrap();

            let message: Message = from_slice(&message_bytes).unwrap();

            send_message(message, &mut stream).await.unwrap();
        });

        let mut client: GenericStream = Box::new(TcpStream::connect(&addr).await?);

        // Create a client that sends a message and instantly receives it
        send_message(message, &mut client).await?;
        let response_bytes = receive_bytes(&mut client).await?;
        let _message: Message = from_slice(&response_bytes)
            .map_err(|err| Error::MessageDeserialization(err.to_string()))?;

        assert_eq!(response_bytes, original_bytes);

        Ok(())
    }
}
