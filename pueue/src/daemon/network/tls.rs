use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use tokio::net::TcpListener;
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig};

use pueue_lib::{
    error::Error,
    network::socket::{GenericStream, Listener, PendingStream},
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
    async fn accept<'a>(&'a self) -> Result<PendingStream, Error> {
        let (stream, _) = self
            .tcp_listener
            .accept()
            .await
            .map_err(|err| Error::IoError("accepting new tcp connection.".to_string(), err))?;

        // Hand the handshake to the caller instead of awaiting it here. A peer that opens a TCP
        // connection and then never sends a ClientHello would otherwise keep us in this function
        // forever, which stops the accept loop from picking up any other connection.
        let acceptor = self.tls_acceptor.clone();
        Ok(Box::pin(async move {
            let tls_stream = acceptor
                .accept(stream)
                .await
                .map_err(|err| Error::IoError("accepting new tls connection.".to_string(), err))?;

            Ok(Box::new(tls_stream) as GenericStream)
        }))
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

/// Load the passed certificates file
pub fn load_certs<'a>(path: &Path) -> Result<Vec<CertificateDer<'a>>, Error> {
    CertificateDer::pem_file_iter(path)
        .map_err(|err| {
            Error::CertificateFailure(format!("Failed to open daemon certificate:\n{err}"))
        })?
        .map(|res| {
            res.map_err(|err| {
                Error::CertificateFailure(format!("Failed to parse certificate: {err}"))
            })
        })
        .collect()
}

/// Load the passed keys file.
/// Only the first key will be used. It should match the certificate.
pub fn load_key<'a>(path: &Path) -> Result<PrivateKeyDer<'a>, Error> {
    // Try to read pkcs8 format first
    PrivateKeyDer::from_pem_file(path).map_err(|err| {
        Error::CertificateFailure(format!("Failed to private key from pem file:\n{err}."))
    })
}
