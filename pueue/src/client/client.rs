use std::io::stdout;

use clap::crate_version;
use crossterm::tty::IsTty;
use pueue_lib::{
    network::{message::*, protocol::*, secret::read_shared_secret},
    settings::Settings,
    Error,
};
use serde::Serialize;

use super::style::OutputStyle;
use crate::{client::cli::ColorChoice, internal_prelude::*};

/// This struct contains the base logic for the client.
/// The client is responsible for connecting to the daemon, sending instructions
/// and interpreting their responses.
///
/// Most commands are a simple ping-pong. However, some commands require a more complex
/// communication pattern, such as the `follow` command, which can read local files,
/// or the `edit` command, which needs to open an editor.
pub struct Client {
    pub settings: Settings,
    pub style: OutputStyle,
    pub stream: GenericStream,
    pub daemon_version: String,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("settings", &self.settings)
            .field("style", &self.style)
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
    pub async fn new(
        settings: Settings,
        show_version_warning: bool,
        color: &ColorChoice,
    ) -> Result<Self> {
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

        // Info if the daemon runs a different version.
        // Backward compatibility should work, but some features might not work as expected.
        if daemon_version != crate_version!() && show_version_warning {
            warn!("Different daemon version detected '{daemon_version}'. Consider restarting the daemon.");
        }

        // Determine whether we should color/style our output or not.
        // The user can explicitly disable/enable this, otherwise we check whether we are on a TTY.
        let style_enabled = match color {
            ColorChoice::Auto => stdout().is_tty(),
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        };
        let style = OutputStyle::new(&settings, style_enabled);

        Ok(Client {
            settings,
            style,
            stream,
            daemon_version,
        })
    }

    /// Convenience function to get a mutable handle on the client's stream.
    pub fn stream(&mut self) -> &mut GenericStream {
        &mut self.stream
    }

    /// Convenience wrapper around [`pueue_lib::send_request`] to directly send [`Request`]s.
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
