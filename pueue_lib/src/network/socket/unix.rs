use std::convert::TryFrom;

use async_trait::async_trait;
use rustls::pki_types::ServerName;
use tokio::net::{TcpStream, UnixListener, UnixStream};

use super::{ConnectionSettings, GenericStream, Listener, Stream, get_tls_connector};
use crate::error::Error;

#[async_trait]
impl Listener for UnixListener {
    async fn accept<'a>(&'a self) -> Result<GenericStream, Error> {
        let (stream, _) = self
            .accept()
            .await
            .map_err(|err| Error::IoError("accepting new unix connection.".to_string(), err))?;
        Ok(Box::new(stream))
    }
}

/// A new trait, which can be used to represent Unix- and Tls encrypted TcpStreams. \
/// This is necessary to write generic functions where both types can be used.
impl Stream for UnixStream {}

/// Get a new stream for the client. \
/// This can either be a UnixStream or a Tls encrypted TCPStream, depending on the parameters.
pub async fn get_client_stream(settings: ConnectionSettings<'_>) -> Result<GenericStream, Error> {
    match settings {
        // Create a unix socket
        ConnectionSettings::UnixSocket { path } => {
            let stream = UnixStream::connect(&path).await.map_err(|err| {
                Error::IoPathError(path, "connecting to daemon. Did you start it?", err)
            })?;

            Ok(Box::new(stream))
        }
        // Connect to the daemon via TCP
        ConnectionSettings::TlsTcpSocket {
            host,
            port,
            certificate,
        } => {
            let address = format!("{host}:{port}");
            let tcp_stream = TcpStream::connect(&address).await.map_err(|_| {
                Error::Connection(format!(
                    "Failed to connect to the daemon on {address}. Did you start it?"
                ))
            })?;

            // Get the configured rustls TlsConnector
            let tls_connector = get_tls_connector(certificate).await.map_err(|err| {
                Error::Connection(format!("Failed to initialize tls connector:\n{err}."))
            })?;

            // Initialize the TLS layer
            let stream = tls_connector
                .connect(ServerName::try_from("pueue.local").unwrap(), tcp_stream)
                .await
                .map_err(|err| Error::Connection(format!("Failed to initialize tls:\n{err}.")))?;

            Ok(Box::new(stream))
        }
    }
}
