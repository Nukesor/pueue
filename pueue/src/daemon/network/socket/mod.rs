//! Socket handling is platform specific code.
//!
//! The submodules of this module represent the different implementations for
//! each supported platform.
//! Depending on the target, the respective platform is read and loaded into this scope.
//!
//! Pueue has a very simple protocol that needs to be followed.
//!
//! [handshake]:
//! 1. Client sends secret for authentication
//! 2. If secret is valid, the daemon sends its own version to the client.
//!
//! [handle_incoming]: Actual handling of application logic
//! 1. The Client sends the instruction message.
//! 2. The Daemon reads the instruction and acts upon it.
//! 3. The Daemon sends a response
//!
//! There're two edge-cases where the [handle_incoming] pattern differs:
//! 1. Shutdown. In that case the message is sent and the daemon shuts down afterwards.
//! 2. Streaming of logs. The Daemon will continuously send messages with log chunks until the
//!    watched task finished or the client disconnects.

use std::time::Duration;

use pueue_lib::{
    Error, PROTOCOL_VERSION, Settings, message::*, network::protocol::*, secret::read_shared_secret,
};
use tokio::time::{Instant, sleep_until, timeout};

use crate::{
    daemon::{internal_state::SharedState, network::message_handler::handle_request},
    internal_prelude::*,
};

/// Shared socket logic
#[cfg_attr(not(target_os = "windows"), path = "unix.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod platform;
pub use self::platform::*;

/// How long a client may take to get from an accepted connection to a valid secret.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// Listen for new connections on the socket.
/// On a new connection, the connected stream will be handled in a separate tokio task.
pub async fn accept_incoming(settings: Settings, state: SharedState) -> Result<()> {
    let listener = get_listener(&settings.shared).await?;
    // Read secret once to prevent multiple disk reads.
    let secret = read_shared_secret(&settings.shared.shared_secret_path())?;

    loop {
        // Poll incoming connections.
        let pending_stream = match listener.accept().await {
            Ok(pending_stream) => pending_stream,
            Err(err) => {
                warn!("Failed connecting to client: {err:?}");
                continue;
            }
        };

        // Start a new task for the request
        let state_clone = state.clone();
        let secret_clone = secret.clone();
        let settings_clone = settings.clone();
        tokio::spawn(async move {
            // To prevent DOS attacks, the authentication handshake is performed with a timeout.
            let stream =
                match timeout(HANDSHAKE_TIMEOUT, handshake(pending_stream, &secret_clone)).await {
                    Ok(Ok(Some(stream))) => stream,
                    // The client hung up before sending anything.
                    Ok(Ok(None)) => return,
                    Ok(Err(err)) => {
                        warn!("Failed to accept client: {err:?}");
                        return;
                    }
                    Err(_) => {
                        warn!("Client didn't complete the handshake within {HANDSHAKE_TIMEOUT:?}.");
                        return;
                    }
                };

            let _result = handle_incoming(stream, state_clone, settings_clone).await;
        });
    }
}

/// Finish establishing a connection and authenticate the client.
///
/// Returns `None` if the client disconnected without sending anything. Those are not reported,
/// as those clients are most likely port scanners and such.
///
/// [accept_incoming] takes care of the timeout handling.
async fn handshake(pending_stream: PendingStream, secret: &[u8]) -> Result<Option<GenericStream>> {
    // Finish the connection in here rather than in the accept loop, so a peer that stalls during
    // the TLS handshake only holds up itself. See [Listener::accept].
    let mut stream = pending_stream.await?;

    // Receive the secret once and check, whether the client is allowed to connect
    // We only allow max payload sizes of 4MB for this one.
    // Daemon's might be exposed publicly and get random traffic, potentially announcing huge
    // payloads that would result in an OOM.
    let payload_bytes =
        receive_bytes_with_max_size(&mut stream, Some(4 * (2usize.pow(20)))).await?;

    // Didn't receive any bytes. The client disconnected.
    if payload_bytes.is_empty() {
        info!("Client went away");
        return Ok(None);
    }

    let start = Instant::now();

    // Return if we got a wrong secret from the client.
    if payload_bytes != *secret {
        // Don't log the payload itself. Anyone can reach this without authenticating, so echoing
        // it would let them write up to 4MB of arbitrary bytes into our log per connection.
        warn!("Received invalid secret of {} bytes.", payload_bytes.len());

        // Always take the same amount of time before closing the socket on a wrong secret, so the
        // comparison above can't that be timed.
        sleep_until(start + Duration::from_secs(1)).await;
        bail!("Received invalid secret");
    }

    // Send confirmation to the client, that the secret was valid.
    // This is also the current version of the pueue_lib protocol used by the daemon,
    // so the client can inform users if the daemon needs a restart in case of a version mismatch.
    send_bytes(PROTOCOL_VERSION.as_bytes(), &mut stream).await?;

    Ok(Some(stream))
}

/// Serve an authenticated client until it goes away.
///
/// See module docs for more information.
pub async fn handle_incoming(
    mut stream: GenericStream,
    state: SharedState,
    settings: Settings,
) -> Result<()> {
    loop {
        // Receive the actual instruction from the client
        let request_result = receive_message(&mut stream).await;

        if let Err(Error::EmptyPayload) = request_result {
            debug!("Client went away");
            return Ok(());
        }

        // In case of a deserialization error, respond the error to the client and return early.
        if let Err(Error::MessageDeserialization(err)) = request_result {
            send_response(
                create_failure_response(format!("Failed to deserialize message: {err}")),
                &mut stream,
            )
            .await?;
            return Ok(());
        }

        let request = request_result?;

        handle_request(&mut stream, request, &state, &settings).await?;
    }
}
