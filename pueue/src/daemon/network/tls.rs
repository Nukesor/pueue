use std::{fs::File, io::BufReader, path::Path, sync::Arc};

use async_trait::async_trait;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{pkcs8_private_keys, rsa_private_keys};
use tokio::net::TcpListener;
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig};

use pueue_lib::{
    error::Error,
    network::socket::{GenericStream, Listener},
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

/// Load the passed certificates file
pub fn load_certs<'a>(path: &Path) -> Result<Vec<CertificateDer<'a>>, Error> {
    let file = File::open(path)
        .map_err(|err| Error::IoPathError(path.to_path_buf(), "opening cert", err))?;
    let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut BufReader::new(file))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .map_err(|_| Error::CertificateFailure("Failed to parse daemon certificate.".into()))?
        .into_iter()
        .collect();

    Ok(certs)
}

/// Load the passed keys file.
/// Only the first key will be used. It should match the certificate.
pub fn load_key<'a>(path: &Path) -> Result<PrivateKeyDer<'a>, Error> {
    let file = File::open(path)
        .map_err(|err| Error::IoPathError(path.to_path_buf(), "opening key", err))?;

    // Try to read pkcs8 format first
    let keys = pkcs8_private_keys(&mut BufReader::new(&file))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .map_err(|_| Error::CertificateFailure("Failed to parse pkcs8 format.".into()));

    if let Ok(keys) = keys {
        if let Some(key) = keys.into_iter().next() {
            return Ok(PrivateKeyDer::Pkcs8(key));
        }
    }

    // Try the normal rsa format afterwards.
    let keys = rsa_private_keys(&mut BufReader::new(file))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .map_err(|_| Error::CertificateFailure("Failed to parse daemon key.".into()))?;

    if let Some(key) = keys.into_iter().next() {
        return Ok(PrivateKeyDer::Pkcs1(key));
    }

    Err(Error::CertificateFailure(format!(
        "Couldn't extract private key from keyfile {path:?}",
    )))
}
