use std::convert::TryFrom;

use rustls::pki_types::ServerName;
use tokio::net::TcpStream;

use super::GenericStream;
use crate::{error::Error, network::tls::get_tls_connector, settings::Shared};

/// Get a new stream for the client.
/// This can either be a UnixStream or a Tls encrypted TCPStream, depending on the parameters.
pub async fn get_client_stream(settings: &Shared) -> Result<GenericStream, Error> {
    // Connect to the daemon via TCP
    let address = format!("{}:{}", settings.host, settings.port);
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
