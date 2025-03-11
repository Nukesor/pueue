use std::convert::TryFrom;

use async_trait::async_trait;
use rustls::pki_types::ServerName;
use tokio::net::{TcpStream, UnixListener, UnixStream};

use super::{GenericStream, Listener, Stream};
use crate::{error::Error, network::tls::get_tls_connector, settings::Shared};

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
pub async fn get_client_stream(settings: &Shared) -> Result<GenericStream, Error> {
    // Create a unix socket, if the config says so.
    if settings.use_unix_socket {
        let unix_socket_path = settings.unix_socket_path();
        let stream = UnixStream::connect(&unix_socket_path)
            .await
            .map_err(|err| {
                Error::IoPathError(
                    unix_socket_path,
                    "connecting to daemon. Did you start it?",
                    err,
                )
            })?;

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
        .map_err(|err| Error::Connection(format!("Failed to initialize tls connector:\n{err}.")))?;

    // Initialize the TLS layer
    let stream = tls_connector
        .connect(ServerName::try_from("pueue.local").unwrap(), tcp_stream)
        .await
        .map_err(|err| Error::Connection(format!("Failed to initialize tls:\n{err}.")))?;

    Ok(Box::new(stream))
}
