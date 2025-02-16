use color_eyre::{
    eyre::{bail, Context},
    Result,
};
use serde::Serialize;

use crate::{
    internal_prelude::*,
    network::{message::*, protocol::*, secret::read_shared_secret},
    settings::Settings,
    Error, PROTOCOL_VERSION,
};

/// This struct contains the base logic for the client.
/// The client is responsible for connecting to the daemon, sending instructions
/// and interpreting their responses.
///
/// ```no_run
/// use pueue_lib::{Settings, Client, Response, Request};
/// # use color_eyre::{Result, eyre::Context};
///
/// # #[tokio::main]
/// # async fn main() -> Result<()> {
///
/// // Read settings from the default configuration file location.
/// let (pueue_settings, _) = Settings::read(&None)?;
///
/// // Create a client. This already establishes a connection to the daemon.
/// let mut client = Client::new(pueue_settings, true)
///     .await
///     .context("Failed to initialize client.")?;
///
/// // Request the state.
/// client.send_request(Request::Status).await?;
/// let response: Response = client.receive_response().await?;
///
/// let _state = match response {
///     Response::Status(state) => state,
///     _ => unreachable!(),
/// };
/// # Ok(())
/// # }
/// ```
pub struct Client {
    pub settings: Settings,
    pub stream: GenericStream,
    pub daemon_version: String,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("settings", &self.settings)
            .field("stream", &"GenericStream<not_debuggable>")
            .finish()
    }
}

impl Client {
    /// Initialize a new client.
    /// This includes establishing a connection to the daemon:
    ///     - Connect to the daemon.
    ///     - Authorize via secret.
    ///     - Check versions incompatibilities.
    ///
    /// If the `show_version_warning` flag is `true` and the daemon has a different version than
    /// the client, a warning will be logged.
    pub async fn new(settings: Settings, show_version_warning: bool) -> Result<Self> {
        // Connect to daemon and get stream used for communication.
        let mut stream = get_client_stream(&settings.shared)
            .await
            .context("Failed to initialize stream.")?;

        // Next we do a handshake with the daemon
        // 1. Client sends the secret to the daemon.
        // 2. If successful, the daemon responds with their version.
        let secret = read_shared_secret(&settings.shared.shared_secret_path())?;
        send_bytes(&secret, &mut stream)
            .await
            .context("Failed to send secret.")?;

        // Receive and parse the response. We expect the daemon's version as UTF-8.
        let version_bytes = receive_bytes(&mut stream)
            .await
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
            warn!("Different protocol version detected '{daemon_version}'. Consider updating and restarting the daemon.");
        }

        Ok(Client {
            settings,
            stream,
            daemon_version,
        })
    }

    /// Convenience function to get a mutable handle on the client's stream.
    pub fn stream(&mut self) -> &mut GenericStream {
        &mut self.stream
    }

    /// Convenience wrapper around [`crate::send_request`] to directly send [`Request`]s.
    pub async fn send_request<T>(&mut self, message: T) -> Result<(), Error>
    where
        T: Into<Request>,
        T: Serialize + std::fmt::Debug,
    {
        send_message::<_, Request>(message, &mut self.stream).await
    }

    /// Convenience wrapper that wraps `receive_message` for [`Response`]s
    pub async fn receive_response(&mut self) -> Result<Response, Error> {
        receive_message::<Response>(&mut self.stream).await
    }

    pub fn daemon_version(&self) -> &String {
        &self.daemon_version
    }
}
