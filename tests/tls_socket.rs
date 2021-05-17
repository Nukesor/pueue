use std::path::PathBuf;

use anyhow::Result;
use async_std::task;
use portpicker::pick_unused_port;
use serde_cbor::de::from_slice;
use serde_cbor::ser::to_vec;
use tempdir::TempDir;

use pueue_lib::network::certificate::create_certificates;
use pueue_lib::network::message::*;
use pueue_lib::network::protocol::*;
use pueue_lib::settings::Shared;

#[async_std::test]
/// This tests whether we can create a listener and client, that communicate via TLS sockets.
async fn test_tls_socket() -> Result<()> {
    better_panic::install();
    // Create a temporary directory used for testing.
    let temp_dir = TempDir::new("pueue_lib").unwrap();
    let temp_dir_path = temp_dir.path().to_path_buf();

    std::fs::create_dir(temp_dir_path.join("certs")).unwrap();

    let shared_settings = Shared {
        pueue_directory: temp_dir_path.clone(),
        #[cfg(not(target_os = "windows"))]
        use_unix_socket: false,
        #[cfg(not(target_os = "windows"))]
        unix_socket_path: PathBuf::new(),
        host: "localhost".to_string(),
        port: pick_unused_port()
            .expect("There should be a free port")
            .to_string(),
        daemon_cert: temp_dir_path.clone().join("certs").join("daemon.cert"),
        daemon_key: temp_dir_path.clone().join("certs").join("daemon.key"),
        shared_secret_path: temp_dir.path().to_path_buf().join("secret"),
    };

    // Create new stub tls certificates/keys in our temp directory
    create_certificates(&shared_settings).unwrap();

    let listener = get_listener(&shared_settings).await.unwrap();
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

    let mut client = get_client_stream(&shared_settings).await.unwrap();

    // Create a client that sends a message and instantly receives it
    send_message(message, &mut client).await.unwrap();
    let response_bytes = receive_bytes(&mut client).await.unwrap();
    let _message: Message = from_slice(&response_bytes).unwrap();

    assert_eq!(response_bytes, original_bytes);

    Ok(())
}
