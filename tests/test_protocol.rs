use ::anyhow::Result;
use ::async_std::net::{TcpListener, TcpStream};
use ::async_std::prelude::*;
use ::async_std::task;

use ::pueue::message::{create_success_message, Message};
use ::pueue::protocol::*;

#[async_std::test]
async fn test_single_huge_payload() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    // The message that should be sent
    let payload = "a".repeat(100_000);
    let message = create_success_message(payload);
    let original_bytes = serde_json::to_string(&message)
        .expect("Failed to serialize message.")
        .into_bytes();

    // Spawn a sub thread that:
    // 1. Accepts a new connection
    // 2. Reads a message
    // 3. Sends the same message back
    task::spawn(async move {
        let mut incoming = listener.incoming();
        let mut socket = incoming.next().await.unwrap().unwrap();
        let message_bytes = receive_bytes(&mut socket).await.unwrap();

        let message = String::from_utf8(message_bytes).unwrap();
        let message: Message = serde_json::from_str(&message).unwrap();

        send_message(&message, &mut socket).await.unwrap();
    });

    let mut client = TcpStream::connect(&addr).await?;
    // Create a client that sends a message and instantly receives it
    send_message(&message, &mut client).await?;
    let response_bytes = receive_bytes(&mut client).await?;
    let message = String::from_utf8(response_bytes.clone())?;
    let _message: Message = serde_json::from_str(&message)?;

    assert_eq!(response_bytes, original_bytes);

    Ok(())
}
