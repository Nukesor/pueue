use std::path::PathBuf;

use anyhow::Result;
use async_std::task;
use serde_cbor::de::from_slice;
use serde_cbor::ser::to_vec;
use tempdir::TempDir;

use pueue_lib::network::message::*;
use pueue_lib::network::protocol::*;
use pueue_lib::settings::Shared;

#[cfg(not(target_os = "windows"))]
#[async_std::test]
/// This tests whether we can create a listener and client, that communicate via unix sockets.
/// This includes the handshake between both parties.
async fn test_unix_socket() -> Result<()> {
    // Create a temporary directory used for testing.
    let tempdir = TempDir::new("pueue_lib")?;
    let shared_settings = Shared {
        pueue_directory: tempdir.path().to_path_buf(),
        use_unix_socket: true,
        unix_socket_path: tempdir.path().to_path_buf().join("test.socket"),
        host: "".to_string(),
        port: "".to_string(),
        daemon_cert: PathBuf::new(),
        daemon_key: PathBuf::new(),
        shared_secret_path: tempdir.into_path().join("secret"),
    };

    let listener = get_listener(&shared_settings).await?;
    let message = create_success_message("This is a test");
    let original_bytes = to_vec(&message).expect("Failed to serialize message.");

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

    let mut client = get_client_stream(&shared_settings).await?;

    // Create a client that sends a message and instantly receives it
    send_message(message, &mut client).await?;
    let response_bytes = receive_bytes(&mut client).await?;
    let _message: Message = from_slice(&response_bytes)?;

    assert_eq!(response_bytes, original_bytes);

    Ok(())
}
