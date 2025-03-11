use ciborium::{from_reader, into_writer};
use color_eyre::Result;
use pretty_assertions::assert_eq;
use tokio::task;

use pueue::daemon::network::{certificate::create_certificates, socket::get_listener};
use pueue_lib::{message::*, network::protocol::*};

use crate::helper::daemon_base_setup;

/// This tests whether we can create a listener and client, that communicate via TLS sockets.
#[tokio::test]
async fn test_tls_socket() -> Result<()> {
    better_panic::install();
    let (settings, _tempdir) = daemon_base_setup()?;

    // Create new stub tls certificates/keys in our temp directory
    create_certificates(&settings.shared).unwrap();

    let listener = get_listener(&settings.shared).await.unwrap();
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

    let mut client = get_client_stream(&settings.shared).await.unwrap();

    // Create a client that sends a message and instantly receives it
    send_request(message, &mut client).await.unwrap();
    let response_bytes = receive_bytes(&mut client).await.unwrap();
    let _message: Request = from_reader(response_bytes.as_slice()).unwrap();

    assert_eq!(response_bytes, original_bytes);

    Ok(())
}
