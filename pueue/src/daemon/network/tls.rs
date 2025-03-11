use std::sync::Arc;

use async_trait::async_trait;
use tokio::net::TcpListener;
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig};

use pueue_lib::{
    error::Error,
    network::{
        socket::{GenericStream, Listener},
        tls::{load_certs, load_key},
    },
    settings::Shared,
};

/// This is a helper struct for TCP connections.
/// TCP should always be used in conjunction with TLS.
/// That's why this helper exists, which encapsulates the logic of accepting a new
/// connection and initializing the TLS layer on top of it.
/// This way we can expose an `accept` function and implement the Listener trait.
pub struct TlsTcpListener {
    pub tcp_listener: TcpListener,
    pub tls_acceptor: TlsAcceptor,
}

#[async_trait]
impl Listener for TlsTcpListener {
    async fn accept<'a>(&'a self) -> Result<GenericStream, Error> {
        let (stream, _) = self
            .tcp_listener
            .accept()
            .await
            .map_err(|err| Error::IoError("accepting new tcp connection.".to_string(), err))?;
        let tls_stream = self
            .tls_acceptor
            .accept(stream)
            .await
            .map_err(|err| Error::IoError("accepting new tls connection.".to_string(), err))?;

        Ok(Box::new(tls_stream))
    }
}

/// Configure the server using rusttls. \
/// A TLS server needs a certificate and a fitting private key.
pub fn get_tls_listener(settings: &Shared) -> Result<TlsAcceptor, Error> {
    // Set the server-side key and certificate that should be used for all communication.
    let certs = load_certs(&settings.daemon_cert())?;
    let key = load_key(&settings.daemon_key())?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| Error::CertificateFailure(format!("Failed to build TLS Acceptor: {err}")))?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}
