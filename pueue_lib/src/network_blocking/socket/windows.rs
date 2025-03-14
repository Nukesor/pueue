use std::net::TcpStream;

use super::{ConnectionSettings, GenericBlockingStream, get_tls_connector};
use crate::error::Error;

/// Get a new stream for the client.
/// This can either be a UnixStream or a Tls encrypted TCPStream, depending on the parameters.
pub fn get_client_stream(settings: ConnectionSettings<'_>) -> Result<GenericBlockingStream, Error> {
    match settings {
        ConnectionSettings::TlsTcpSocket {
            host,
            port,
            certificate,
        } => {
            // Connect to the daemon via TCP
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
