use std::convert::TryFrom;

use rustls::pki_types::ServerName;
use tokio::net::TcpStream;

use super::{ConnectionSettings, GenericStream, get_tls_connector};
use crate::error::Error;

/// Get a new stream for the client.
/// This can either be a UnixStream or a Tls encrypted TCPStream, depending on the parameters.
pub async fn get_client_stream(settings: ConnectionSettings<'_>) -> Result<GenericStream, Error> {
    match settings {
        ConnectionSettings::TlsTcpSocket {
            host,
            port,
            certificate,
        } => {
            // Connect to the daemon via TCP
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
                .map_err(|err| Error::Connection(format!("Failed to initialize tls {err}.")))?;

            Ok(Box::new(stream))
        }
    }
}
