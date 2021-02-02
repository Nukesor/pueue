use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use async_std::io::{Read, Write};
use async_std::net::{TcpListener, TcpStream};
use async_std::os::unix::net::{UnixListener, UnixStream};
use async_tls::TlsAcceptor;
use async_trait::async_trait;

use crate::network::tls::{get_tls_connector, get_tls_listener};
use crate::settings::Shared;

/// Unix specific cleanup handling when getting a SIGINT/SIGTERM.
pub fn socket_cleanup(settings: &Shared) {
    // Clean up the unix socket if we're using it and it exists.
    if settings.use_unix_socket && PathBuf::from(&settings.unix_socket_path).exists() {
        std::fs::remove_file(&settings.unix_socket_path)
            .expect("Failed to remove unix socket on shutdown");
    }
}

/// A new trait, which can be used to represent Unix- and TcpListeners. \
/// This is necessary to easily write generic functions where both types can be used.
#[async_trait]
pub trait GenericListener: Sync + Send {
    async fn accept<'a>(&'a self) -> Result<GenericStream>;
}

/// This is a helper struct for TCP connections.
/// TCP should always be used in conjunction with TLS.
/// That's why this helper exists, which encapsulates the logic of accepting a new
/// connection and initializing the TLS layer on top of it.
/// This way we can expose an `accept` function and implement the GenericListener trait.
pub(crate) struct TlsTcpListener {
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
        let (stream, _) = self.accept().await?;
        Ok(Box::new(stream))
    }
}

/// A new trait, which can be used to represent Unix- and Tls encrypted TcpStreams. \
/// This is necessary to write generic functions where both types can be used.
pub trait Stream: Read + Write + Unpin + Send {}
impl Stream for UnixStream {}
impl Stream for async_tls::server::TlsStream<TcpStream> {}
impl Stream for async_tls::client::TlsStream<TcpStream> {}

/// Convenience type, so we don't have type write Box<dyn GenericListener> all the time.
pub type Listener = Box<dyn GenericListener>;
/// Convenience type, so we don't have type write Box<dyn Stream> all the time. \
/// This also prevents name collisions, since `Stream` is imported in many preludes.
pub type GenericStream = Box<dyn Stream>;

/// Get a new stream for the client. \
/// This can either be a UnixStream or a Tls encrypted TCPStream, depending on the parameters.
pub async fn get_client_stream(settings: &Shared) -> Result<GenericStream> {
    // Create a unix socket, if the config says so.
    if settings.use_unix_socket {
        if !PathBuf::from(&settings.unix_socket_path).exists() {
            bail!(
                "Couldn't find unix socket at path {:?}. Is the daemon running yet?",
                &settings.unix_socket_path
            );
        }
        let stream = UnixStream::connect(&settings.unix_socket_path).await?;
        return Ok(Box::new(stream));
    }

    // Connect to the daemon via TCP
    let address = format!("{}:{}", &settings.host, &settings.port);
    let tcp_stream = TcpStream::connect(&address).await.context(format!(
        "Failed to connect to the daemon on {}. Did you start it?",
        &address
    ))?;

    // Get the configured rustls TlsConnector
    let tls_connector = get_tls_connector(&settings)
        .await
        .context("Failed to initialize TLS Connector")?;

    // Initialize the TLS layer
    let stream = tls_connector
        .connect("pueue.local", tcp_stream)
        .await
        .context("Failed to initialize TLS stream")?;

    Ok(Box::new(stream))
}

/// Get a new listener for the daemon. \
/// This can either be a UnixListener or a TCPlistener, depending on the parameters.
pub async fn get_listener(settings: &Shared) -> Result<Listener> {
    if settings.use_unix_socket {
        // Check, if the socket already exists
        // In case it does, we have to check, if it's an active socket.
        // If it is, we have to throw an error, because another daemon is already running.
        // Otherwise, we can simply remove it.
        if PathBuf::from(&settings.unix_socket_path).exists() {
            if get_client_stream(&settings).await.is_ok() {
                bail!(
                    "There seems to be an active pueue daemon.\n\
                      If you're sure there isn't, please remove the socket by hand \
                      inside the pueue_directory."
                );
            }

            std::fs::remove_file(&settings.unix_socket_path)?;
        }

        return Ok(Box::new(
            UnixListener::bind(&settings.unix_socket_path).await?,
        ));
    }

    // This is the listener, which accepts low-level TCP connections
    let address = format!("{}:{}", &settings.host, &settings.port);
    let tcp_listener = TcpListener::bind(&address)
        .await
        .context(format!("Failed to listen on address: {}", address))?;

    // This is the TLS acceptor, which initializes the TLS layer
    let tls_acceptor = get_tls_listener(&settings)?;

    // Create a struct, which accepts connections and initializes a TLS layer in one go.
    let tls_listener = TlsTcpListener {
        tcp_listener,
        tls_acceptor,
    };

    Ok(Box::new(tls_listener))
}
