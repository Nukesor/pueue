use std::io::Cursor;
use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ciborium::{from_reader, into_writer};
use serde::{Serialize, de::DeserializeOwned};

// Reexport all stream/socket related stuff for convenience purposes
pub use super::socket::*;
use crate::{
    error::Error,
    internal_prelude::*,
    message::{request::Request, response::Response},
};

// We choose a packet size of 1280 to be on the safe site regarding IPv6 MTU.
pub const PACKET_SIZE: usize = 1280;

/// Convenience wrapper around `send_message` to directly send [`Request`]s.
pub fn send_request<T>(message: T, stream: &mut GenericBlockingStream) -> Result<(), Error>
where
    T: Into<Request>,
    T: Serialize + std::fmt::Debug,
{
    send_message::<_, Request>(message, stream)
}

/// Convenience wrapper around `send_message` to directly send [`Response`]s.
pub fn send_response<T>(message: T, stream: &mut GenericBlockingStream) -> Result<(), Error>
where
    T: Into<Response>,
    T: Serialize + std::fmt::Debug,
{
    send_message::<_, Response>(message, stream)
}

/// Convenience wrapper around send_bytes.
/// Deserialize a message and feed the bytes into send_bytes.
///
/// This function is designed to be used with the inner values of the `Request`
/// or `Response` enums.
/// If there's no inner variant, you might need to anotate the type:
/// `send_message::<_, Request>(Request::Status, &mut stream)`
pub fn send_message<O, T>(message: O, stream: &mut GenericBlockingStream) -> Result<(), Error>
where
    O: Into<T>,
    T: Serialize + std::fmt::Debug,
{
    let message: T = message.into();
    debug!("Sending message: {message:#?}",);
    // Prepare command for transfer and determine message byte size
    let mut payload = Vec::new();
    into_writer(&message, &mut payload)
        .map_err(|err| Error::MessageSerialization(err.to_string()))?;

    send_bytes(&payload, stream)
}

/// Send a Vec of bytes.
/// This is part of the basic protocol beneath all communication. \
///
/// 1. Sends a u64 as 4bytes in BigEndian mode, which tells the receiver the length of the payload.
/// 2. Send the payload in chunks of [PACKET_SIZE] bytes.
pub fn send_bytes(payload: &[u8], stream: &mut GenericBlockingStream) -> Result<(), Error> {
    let message_size = payload.len() as u64;

    let mut header = Vec::new();
    WriteBytesExt::write_u64::<BigEndian>(&mut header, message_size).unwrap();

    // Send the request size header first.
    // Afterwards send the request.
    stream
        .write_all(&header)
        .map_err(|err| Error::IoError("sending request size header".to_string(), err))?;

    // Split the payload into 1.4Kbyte chunks
    // 1.5Kbyte is the MUT for TCP, but some carrier have a little less, such as Wireguard.
    for chunk in payload.chunks(PACKET_SIZE) {
        stream
            .write_all(chunk)
            .map_err(|err| Error::IoError("sending payload chunk".to_string(), err))?;
    }

    stream.flush()?;

    Ok(())
}

pub fn receive_bytes(stream: &mut GenericBlockingStream) -> Result<Vec<u8>, Error> {
    receive_bytes_with_max_size(stream, None)
}

/// Receive a byte stream. \
/// This is part of the basic protocol beneath all communication. \
///
/// 1. First of, the client sends a u64 as a 4byte vector in BigEndian mode, which specifies the
///    length of the payload we're going to receive.
/// 2. Receive chunks of [PACKET_SIZE] bytes until we finished all expected bytes.
pub fn receive_bytes_with_max_size(
    stream: &mut GenericBlockingStream,
    max_size: Option<usize>,
) -> Result<Vec<u8>, Error> {
    // Receive the header with the overall message size
    let mut header = vec![0; 8];
    stream
        .read_exact(&mut header)
        .map_err(|err| Error::IoError("reading request size header".to_string(), err))?;
    let mut header = Cursor::new(header);
    let message_size = ReadBytesExt::read_u64::<BigEndian>(&mut header)? as usize;

    if let Some(max_size) = max_size
        && message_size > max_size
    {
        error!("Client requested message size of {message_size}, but only {max_size} is allowed.");
        return Err(Error::MessageTooBig(message_size, max_size));
    }

    // Show a warning if we see unusually large payloads. In this case payloads that're bigger than
    // 20MB, which is pretty large considering pueue is usually only sending a bit of text.
    if message_size > (20 * (2usize.pow(20))) {
        warn!("Client is sending a large payload: {message_size} bytes.");
    }

    // Buffer for the whole payload
    let mut payload_bytes = Vec::with_capacity(message_size);

    // Receive chunks until we reached the expected message size
    while payload_bytes.len() < message_size {
        let remaining_bytes = message_size - payload_bytes.len();
        let mut chunk_buffer: Vec<u8> = if remaining_bytes < PACKET_SIZE {
            // The remaining bytes fit into less then our PACKET_SIZE.
            // In this case, we have to be exact to prevent us from accidentally reading bytes
            // of the next message that might already be in the queue.
            vec![0; remaining_bytes]
        } else {
            // Create a static buffer with our max packet size.
            vec![0; PACKET_SIZE]
        };

        // Read data and get the amount of received bytes
        let received_bytes = stream
            .read(&mut chunk_buffer)
            .map_err(|err| Error::IoError("reading next chunk".to_string(), err))?;

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

/// Convenience wrapper that wraps `receive_message` for [`Request`]s
pub fn receive_request(stream: &mut GenericBlockingStream) -> Result<Request, Error> {
    receive_message::<Request>(stream)
}

/// Convenience wrapper that wraps `receive_message` for [`Response`]s
pub fn receive_response(stream: &mut GenericBlockingStream) -> Result<Response, Error> {
    receive_message::<Response>(stream)
}

/// Convenience wrapper that receives a message and converts it into `T`.
pub fn receive_message<T: DeserializeOwned + std::fmt::Debug>(
    stream: &mut GenericBlockingStream,
) -> Result<T, Error> {
    let payload_bytes = receive_bytes(stream)?;
    if payload_bytes.is_empty() {
        return Err(Error::EmptyPayload);
    }

    // Deserialize the message.
    let message: T = from_reader(payload_bytes.as_slice()).map_err(|err| {
        // In the case of an error, try to deserialize it to a generic cbor Value.
        // That way we know whether the payload was corrupted or maybe just unexpected due to
        // version differences.
        if let Ok(value) = from_reader::<ciborium::Value, _>(payload_bytes.as_slice()) {
            Error::UnexpectedPayload(value)
        } else {
            Error::MessageDeserialization(err.to_string())
        }
    })?;
    debug!("Received message: {message:#?}");

    Ok(message)
}

#[cfg(test)]
mod test {
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::time::Duration;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::message::request::{Request, SendRequest};
    use crate::network_blocking::socket::BlockingStream;

    // Implement generic Listener/Stream traits, so we can test stuff on normal TCP
    impl BlockingListener for TcpListener {
        fn accept(&self) -> Result<GenericBlockingStream, Error> {
            let (stream, _) = self.accept()?;
            Ok(Box::new(stream))
        }
    }
    impl BlockingStream for TcpStream {}

    #[test]
    fn test_single_huge_payload() -> Result<(), Error> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;

        // The message that should be sent
        let payload = "a".repeat(100_000);
        let request: Request = SendRequest {
            task_id: 0,
            input: payload,
        }
        .into();
        let mut original_bytes = Vec::new();
        into_writer(&request, &mut original_bytes).expect("Failed to serialize message.");

        let listener: GenericBlockingListener = Box::new(listener);

        // Spawn a sub thread that:
        // 1. Accepts a new connection
        // 2. Reads a message
        // 3. Sends the same message back
        thread::spawn(move || {
            let mut stream = listener.accept().unwrap();
            let message_bytes = receive_bytes(&mut stream).unwrap();

            let message: Request = from_reader(message_bytes.as_slice()).unwrap();

            send_request(message, &mut stream).unwrap();
        });

        let mut client: GenericBlockingStream = Box::new(TcpStream::connect(addr)?);

        // Create a client that sends a message and instantly receives it
        send_request(request, &mut client)?;
        let response_bytes = receive_bytes(&mut client)?;
        let _message: Request = from_reader(response_bytes.as_slice())
            .map_err(|err| Error::MessageDeserialization(err.to_string()))?;

        assert_eq!(response_bytes, original_bytes);

        Ok(())
    }

    /// Test that multiple messages can be sent by a sender.
    /// The receiver must be able to handle those massages, even if multiple are in the buffer
    /// at once.
    #[test]
    fn test_successive_messages() -> Result<(), Error> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;

        let listener: GenericBlockingListener = Box::new(listener);

        // Spawn a sub thread that:
        // 1. Accepts a new connection.
        // 2. Immediately sends two messages in quick succession.
        thread::spawn(move || {
            let mut stream = listener.accept().unwrap();

            send_request(Request::Status, &mut stream).unwrap();
            send_request(Request::Remove(vec![0, 2, 3]), &mut stream).unwrap();
        });

        // Create a receiver stream
        let mut client: GenericBlockingStream = Box::new(TcpStream::connect(addr)?);
        // Wait for a short time to allow the sender to send all messages
        std::thread::sleep(Duration::from_millis(500));

        // Get both individual messages that have been sent.
        let message_a = receive_message(&mut client).expect("First message");
        let message_b = receive_message(&mut client).expect("Second message");

        assert_eq!(Request::Status, message_a);
        assert_eq!(Request::Remove(vec![0, 2, 3]), message_b);

        Ok(())
    }

    /// Ensure there's no OOM if a huge payload during the handshake phase is being requested.
    ///
    /// We limit the receiving buffer to ~4MB for the incoming secret to prevent (potentially
    /// unintended) DoS attacks when something connect to Pueue and sends a malformed secret
    /// payload.
    #[test]
    fn test_restricted_payload_size() -> Result<(), Error> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;

        let listener: GenericBlockingListener = Box::new(listener);

        // Spawn a sub thread that:
        // 1. Accepts a new connection.
        // 2. Sends a malformed payload.
        thread::spawn(move || {
            let mut stream = listener.accept().unwrap();

            // Send a payload of 9 bytes to the daemon receiver.
            // The first 8 bytes determine the payload size in BigEndian.
            // This payload requests 2^64 bytes of memory for the secret.
            stream.write_all(&[128, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        });

        // Create a receiver stream
        let mut client: GenericBlockingStream = Box::new(TcpStream::connect(addr)?);
        // Wait for a short time to allow the sender to send the message
        std::thread::sleep(Duration::from_millis(500));

        // Get the message while restricting the payload size to 4MB
        let result = receive_bytes_with_max_size(&mut client, Some(4 * 2usize.pow(20)));

        assert!(
            result.is_err(),
            "The payload should be rejected due to large size"
        );

        Ok(())
    }
}
