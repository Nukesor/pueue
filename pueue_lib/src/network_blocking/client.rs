//! A reference implementation of a simple client that you may use.
//! En/disable via the `client` feature.
use color_eyre::{
    Result,
    eyre::{Context, bail},
};
use serde::Serialize;

use super::protocol::*;
use crate::{Error, PROTOCOL_VERSION, internal_prelude::*, message::*};

/// This struct contains the base logic for the client.
/// The client is responsible for connecting to the daemon, sending instructions
/// and interpreting their responses.
///
/// ```no_run
/// use std::path::PathBuf;
/// use pueue_lib::{
///     BlockingClient,
///     Response,
///     Request,
///     network_blocking::socket::ConnectionSettings,
/// };
/// # use color_eyre::{Result, eyre::Context};
///
/// # #[cfg(target_os = "windows")]
/// # fn main() {}
///
/// # #[cfg(not(target_os = "windows"))]
/// # fn main() -> Result<()> {
///
/// // Connection settings and secret to connect to the daemon.
/// let settings = ConnectionSettings::UnixSocket {
///     path: PathBuf::from("/home/user/.local/share/pueue/pueue.socket"),
/// };
/// let secret = "My secret";
///
/// // Create a client. This already establishes a connection to the daemon.
/// let mut client = BlockingClient::new(settings, secret.as_bytes(), true)
///     .context("Failed to initialize client.")?;
///
/// // Request the state.
/// client.send_request(Request::Status)?;
/// let response: Response = client.receive_response()?;
///
/// let _state = match response {
///     Response::Status(state) => state,
///     _ => unreachable!(),
/// };
/// # Ok(())
/// # }
/// ```
pub struct BlockingClient {
    pub stream: GenericBlockingStream,
    pub daemon_version: String,
}

impl std::fmt::Debug for BlockingClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("stream", &"GenericStream<not_debuggable>")
            .finish()
    }
}

impl BlockingClient {
    /// Initialize a new client.
    /// This includes establishing a connection to the daemon:
    ///     - Connect to the daemon.
    ///     - Authorize via secret.
    ///     - Check versions incompatibilities.
    ///
    /// If the `show_version_warning` flag is `true` and the daemon has a different version than
    /// the client, a warning will be logged.
    pub fn new(
        settings: ConnectionSettings<'_>,
        secret: &[u8],
        show_version_warning: bool,
    ) -> Result<Self> {
        // Connect to daemon and get stream used for communication.
        let mut stream = get_client_stream(settings).context("Failed to initialize stream.")?;

        // Next we do a handshake with the daemon
        // 1. Client sends the secret to the daemon.
        // 2. If successful, the daemon responds with their version.
        send_bytes(secret, &mut stream).context("Failed to send secret.")?;

        // Receive and parse the response. We expect the daemon's version as UTF-8.
        let version_bytes = receive_bytes(&mut stream)
            .context("Failed to receive version during handshake with daemon.")?;
        if version_bytes.is_empty() {
            bail!("Daemon went away after sending secret. Did you use the correct secret?")
        }
        let daemon_version = match String::from_utf8(version_bytes) {
            Ok(version) => version,
            Err(_) => {
                bail!("Daemon sent invalid UTF-8. Did you use the correct secret?")
            }
        };

        // Info if the daemon runs a different protocol version.
        // Backward compatibility should work, but some features might not work as expected.
        if daemon_version != PROTOCOL_VERSION && show_version_warning {
            warn!(
                "Different protocol version detected '{daemon_version}'. Consider updating and restarting the daemon."
            );
        }

        Ok(BlockingClient {
            stream,
            daemon_version,
        })
    }

    /// Convenience function to get a mutable handle on the client's stream.
    pub fn stream(&mut self) -> &mut GenericBlockingStream {
        &mut self.stream
    }

    /// Convenience wrapper around [`super::send_request`] to directly send [`Request`]s.
    pub fn send_request<T>(&mut self, message: T) -> Result<(), Error>
    where
        T: Into<Request>,
        T: Serialize + std::fmt::Debug,
    {
        send_message::<_, Request>(message, &mut self.stream)
    }

    /// Convenience wrapper that wraps `receive_message` for [`Response`]s
    pub fn receive_response(&mut self) -> Result<Response, Error> {
        receive_message::<Response>(&mut self.stream)
    }

    pub fn daemon_version(&self) -> &String {
        &self.daemon_version
    }
}
