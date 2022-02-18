use std::convert::TryFrom;
use std::path::PathBuf;

use async_trait::async_trait;
use rustls::ServerName;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};
use tokio_rustls::TlsAcceptor;

use crate::error::Error;
use crate::network::tls::{get_tls_connector, get_tls_listener};
use crate::settings::Shared;

/// Unix specific cleanup handling when getting a SIGINT/SIGTERM.
pub fn socket_cleanup(settings: &Shared) -> Result<(), std::io::Error> {
    // Clean up the unix socket if we're using it and it exists.
    if settings.use_unix_socket && PathBuf::from(&settings.unix_socket_path()).exists() {
        std::fs::remove_file(&settings.unix_socket_path())?;
    }

    Ok(())
}

/// A new trait, which can be used to represent Unix- and TcpListeners. \
/// This is necessary to easily write generic functions where both types can be used.
#[async_trait]
pub trait Listener: Sync + Send {
    async fn accept<'a>(&'a self) -> Result<GenericStream, Error>;
}

/// This is a helper struct for TCP connections.
/// TCP should always be used in conjunction with TLS.
/// That's why this helper exists, which encapsulates the logic of accepting a new
/// connection and initializing the TLS layer on top of it.
/// This way we can expose an `accept` function and implement the Listener trait.
pub(crate) struct TlsTcpListener {
    tcp_listener: TcpListener,
    tls_acceptor: TlsAcceptor,
}

#[async_trait]
impl Listener for TlsTcpListener {
    async fn accept<'a>(&'a self) -> Result<GenericStream, Error> {
        let (stream, _) = self.tcp_listener.accept().await?;
        Ok(Box::new(self.tls_acceptor.accept(stream).await?))
    }
}

#[async_trait]
impl Listener for UnixListener {
    async fn accept<'a>(&'a self) -> Result<GenericStream, Error> {
        let (stream, _) = self.accept().await?;
        Ok(Box::new(stream))
    }
}

/// A new trait, which can be used to represent Unix- and Tls encrypted TcpStreams. \
/// This is necessary to write generic functions where both types can be used.
pub trait Stream: AsyncRead + AsyncWrite + Unpin + Send {}
impl Stream for UnixStream {}
impl Stream for tokio_rustls::server::TlsStream<TcpStream> {}
impl Stream for tokio_rustls::client::TlsStream<TcpStream> {}

/// Convenience type, so we don't have type write `Box<dyn Listener>` all the time.
pub type GenericListener = Box<dyn Listener>;
/// Convenience type, so we don't have type write `Box<dyn Stream>` all the time. \
/// This also prevents name collisions, since `Stream` is imported in many preludes.
pub type GenericStream = Box<dyn Stream>;

/// Get a new stream for the client. \
/// This can either be a UnixStream or a Tls encrypted TCPStream, depending on the parameters.
pub async fn get_client_stream(settings: &Shared) -> Result<GenericStream, Error> {
    // Create a unix socket, if the config says so.
    if settings.use_unix_socket {
        let unix_socket_path = settings.unix_socket_path();
        if !PathBuf::from(&unix_socket_path).exists() {
            return Err(Error::FileNotFound(format!(
                "Unix socket at path {unix_socket_path:?}. Is the daemon started?",
            )));
        }
        let stream = UnixStream::connect(&unix_socket_path).await?;
        return Ok(Box::new(stream));
    }

    // Connect to the daemon via TCP
    let address = format!("{}:{}", &settings.host, &settings.port);
    let tcp_stream = TcpStream::connect(&address).await.map_err(|_| {
        Error::Connection(format!(
            "Failed to connect to the daemon on {address}. Did you start it?"
        ))
    })?;

    // Get the configured rustls TlsConnector
    let tls_connector = get_tls_connector(settings)
        .await
        .map_err(|err| Error::Connection(format!("Failed to initialize tls connector {err}.")))?;

    // Initialize the TLS layer
    let stream = tls_connector
        .connect(ServerName::try_from("pueue.local").unwrap(), tcp_stream)
        .await
        .map_err(|err| Error::Connection(format!("Failed to initialize tls {err}.")))?;

    Ok(Box::new(stream))
}

/// Get a new listener for the daemon. \
/// This can either be a UnixListener or a TCPlistener, depending on the parameters.
pub async fn get_listener(settings: &Shared) -> Result<GenericListener, Error> {
    if settings.use_unix_socket {
        // Check, if the socket already exists
        // In case it does, we have to check, if it's an active socket.
        // If it is, we have to throw an error, because another daemon is already running.
        // Otherwise, we can simply remove it.
        if PathBuf::from(&settings.unix_socket_path()).exists() {
            if get_client_stream(settings).await.is_ok() {
                return Err(Error::UnixSocketExists);
            }

            std::fs::remove_file(&settings.unix_socket_path())?;
        }

        return Ok(Box::new(UnixListener::bind(&settings.unix_socket_path())?));
    }

    // This is the listener, which accepts low-level TCP connections
    let address = format!("{}:{}", &settings.host, &settings.port);
    let tcp_listener = TcpListener::bind(&address).await?;

    // This is the TLS acceptor, which initializes the TLS layer
    let tls_acceptor = get_tls_listener(settings)?;

    // Create a struct, which accepts connections and initializes a TLS layer in one go.
    let tls_listener = TlsTcpListener {
        tcp_listener,
        tls_acceptor,
    };

    Ok(Box::new(tls_listener))
}
