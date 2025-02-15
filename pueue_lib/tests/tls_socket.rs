use ciborium::{from_reader, into_writer};
use color_eyre::Result;
use pretty_assertions::assert_eq;
use pueue_lib::network::{certificate::create_certificates, message::*, protocol::*};
use tokio::task;

mod helper;

/// This tests whether we can create a listener and client, that communicate via TLS sockets.
#[tokio::test]
async fn test_tls_socket() -> Result<()> {
    better_panic::install();
    let (shared_settings, _tempdir) = helper::get_shared_settings(false);

    // Create new stub tls certificates/keys in our temp directory
    create_certificates(&shared_settings).unwrap();

    let listener = get_listener(&shared_settings).await.unwrap();
    let message = Request::Status;
    let mut original_bytes = Vec::new();
    into_writer(&message, &mut original_bytes).expect("Failed to serialize message.");

    // Spawn a sub thread that:
    // 1. Accepts a new connection
    // 2. Reads a message
    // 3. Sends the same message back
    task::spawn(async move {
        let mut stream = listener.accept().await.unwrap();
        let message_bytes = receive_bytes(&mut stream).await.unwrap();

        let message: Request = from_reader(message_bytes.as_slice()).unwrap();

        send_request(message, &mut stream).await.unwrap();
    });

    let mut client = get_client_stream(&shared_settings).await.unwrap();

    // Create a client that sends a message and instantly receives it
    send_request(message, &mut client).await.unwrap();
    let response_bytes = receive_bytes(&mut client).await.unwrap();
    let _message: Request = from_reader(response_bytes.as_slice()).unwrap();

    assert_eq!(response_bytes, original_bytes);

    Ok(())
}
