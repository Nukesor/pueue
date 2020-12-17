use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use async_std::io::{Read, Write};
use async_std::net::{TcpListener, TcpStream};
use async_std::os::unix::net::{UnixListener, UnixStream};
use async_tls::TlsAcceptor;
use async_trait::async_trait;

use crate::settings::Settings;
use crate::state::SharedState;
use crate::tls::{get_client_tls_connector, load_config};

/// A new trait, which can be used to represent Unix- and TcpListeners.
/// This is necessary to easily write generic functions where both types can be used.
#[async_trait]
pub trait GenericListener: Sync + Send {
    async fn accept<'a>(&'a self) -> Result<GenericStream>;
}

/// This is a helper struct for TCP connections.
/// TCP should always be used in conjunction with TLS.
/// That's why we create a intermediate struct, which will encapsulate the logic of accepting a new
/// connection and initializing the TLS layer on top of it.
/// To the outside, it will just look like another GenericListener
pub struct TlsTcpListener {
    tcp_listener: TcpListener,
    tls_acceptor: TlsAcceptor,
}

#[async_trait]
impl GenericListener for TlsTcpListener {
    async fn accept<'a>(&'a self) -> Result<GenericStream> {
        let (stream, _) = self.tcp_listener.accept().await?;
        Ok(Box::new(self.tls_acceptor.accept(stream).await?))
    }
}

#[async_trait]
impl GenericListener for UnixListener {
    async fn accept<'a>(&'a self) -> Result<GenericStream> {
        let (socket, _) = self.accept().await?;
        Ok(Box::new(socket))
    }
}

/// A new trait, which can be used to represent Unix- and Tls encrypted TcpStreams.
/// This is necessary to write generic functions where both types can be used.
pub trait Stream: Read + Write + Unpin + Send {}
impl Stream for UnixStream {}
impl Stream for async_tls::server::TlsStream<TcpStream> {}
impl Stream for async_tls::client::TlsStream<TcpStream> {}

/// Two convenient types, so we don't have type write Box<dyn ...> all the time.
pub type Listener = Box<dyn GenericListener>;
pub type GenericStream = Box<dyn Stream>;

/// Get a new stream for the client.
/// This can either be a UnixStream or a Tls encrypted TCPStream, depending on the parameters.
pub async fn get_client_socket(
    settings: &Settings,
    cli_port: Option<String>,
    cli_unix_socket_path: Option<String>,
) -> Result<GenericStream> {
    // Get the unix socket path.
    // Commandline arguments take prescedence
    let unix_socket_path = if let Some(path) = cli_unix_socket_path {
        path
    } else {
        settings.shared.unix_socket_path.clone()
    };

    // Get the host for TCP connections.
    // Commandline arguments take prescedence
    // let host = if let Some(host) = cli_host {
    //     host
    // } else {
    //     settings.shared.host.clone()
    // };
    // Don't allow anything else than loopback until we have proper crypto
    let host = "127.0.0.1";

    // Get the host's port for TCP connections.
    // Get the port Commandline arguments take prescedence
    let port = if let Some(port) = cli_port {
        port
    } else {
        settings.shared.port.clone()
    };

    // Create a unix socket, if the config says so.
    if settings.shared.use_unix_socket {
        if !PathBuf::from(&unix_socket_path).exists() {
            bail!(
                "Couldn't find unix socket at path {:?}. Is the daemon running yet?",
                unix_socket_path
            );
        }
        let stream = UnixStream::connect(unix_socket_path).await?;
        return Ok(Box::new(stream));
    }

    // Connect to the daemon via TCP
    let address = format!("{}:{}", &host, &port);
    let tcp_stream = TcpStream::connect(&address).await.context(format!(
        "Failed to connect to the daemon on {}. Did you start it?",
        &address
    ))?;

    // Initialize the TLS layer
    let tls_connector = get_client_tls_connector(&settings)
        .await
        .context("Failed to initialize TLS Connector")?;

    let stream = tls_connector
        .connect("localhost", tcp_stream)
        .await
        .context("Failed to initialize TLS stream")?;

    Ok(Box::new(stream))
}

/// Get a new listener for the daemon.
/// This can either be a UnixListener or a TCPlistener,
/// which depends on the parameters.
pub async fn get_listener(state: &SharedState, cli_port: Option<String>) -> Result<Listener> {
    let state = state.lock().unwrap();
    let (unix_socket_path, tcp_info) = {
        let shared = &state.settings.shared;

        // Return the unix socket path, if we're supposed to use it.
        if shared.use_unix_socket {
            (Some(shared.unix_socket_path.clone()), None)
        } else {
            // Otherwise use tcp sockets on a given port and host.
            // Commandline argument overwrites the configuration files values for port.
            // This also initializes the TLS acceptor.
            let port = if let Some(port) = cli_port {
                port
            } else {
                shared.port.clone()
            };

            let config = load_config(&state.settings)?;
            let acceptor = TlsAcceptor::from(Arc::new(config));
            (None, Some((port, acceptor)))
        }
    };

    if let Some(socket_path) = unix_socket_path {
        // Check, if the socket already exists
        // In case it does, we have to check, if it's an active socket.
        // If it is, we have to throw an error, because another daemon is already running.
        // Otherwise, we can simply remove it.
        if PathBuf::from(&socket_path).exists() {
            if get_client_socket(&state.settings, None, Some(socket_path.clone()))
                .await
                .is_ok()
            {
                bail!(
                    "There seems to be an active pueue daemon.\n\
                      If you're sure there isn't, please remove the socket by hand \
                      inside the pueue_directory."
                );
            }

            std::fs::remove_file(&socket_path)?;
        }

        return Ok(Box::new(UnixListener::bind(socket_path).await?));
    }

    let (port, tls_acceptor) = tcp_info.unwrap();
    let address = format!("127.0.0.1:{}", port);
    let tcp_listener = TcpListener::bind(address).await?;

    // Create a list, which accepts connections and initializes a TLS layer.
    let tls_listener = TlsTcpListener {
        tcp_listener,
        tls_acceptor,
    };

    Ok(Box::new(tls_listener))
}
