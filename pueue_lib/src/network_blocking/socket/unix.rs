use std::net::TcpStream;
use std::os::unix::net::{UnixListener, UnixStream};

use super::{
    BlockingListener, BlockingStream, ConnectionSettings, GenericBlockingStream, get_tls_connector,
};
use crate::error::Error;

impl BlockingListener for UnixListener {
    fn accept(&self) -> Result<GenericBlockingStream, Error> {
        let (stream, _) = self
            .accept()
            .map_err(|err| Error::IoError("accepting new unix connection.".to_string(), err))?;
        Ok(Box::new(stream))
    }
}

/// A new trait, which can be used to represent Unix- and Tls encrypted TcpStreams. \
/// This is necessary to write generic functions where both types can be used.
impl BlockingStream for UnixStream {}

/// Get a new stream for the client. \
/// This can either be a UnixStream or a Tls encrypted TCPStream, depending on the parameters.
pub fn get_client_stream(settings: ConnectionSettings<'_>) -> Result<GenericBlockingStream, Error> {
    match settings {
        // Create a unix socket
        ConnectionSettings::UnixSocket { path } => {
            let stream = UnixStream::connect(&path).map_err(|err| {
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
            let tcp_stream = TcpStream::connect(&address).map_err(|_| {
                Error::Connection(format!(
                    "Failed to connect to the daemon on {address}. Did you start it?"
                ))
            })?;

            // Get the configured rustls TlsConnector
            let tls_connector = get_tls_connector(certificate).map_err(|err| {
                Error::Connection(format!("Failed to initialize tls connector:\n{err}."))
            })?;

            // Initialize the TLS layer
            let stream = tls_connector
                .connect("pueue.local", tcp_stream)
                .map_err(|err| Error::Connection(format!("Failed to initialize tls:\n{err}.")))?;

            Ok(Box::new(stream))
        }
    }
}
