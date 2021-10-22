use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use tokio_rustls::{TlsAcceptor, TlsConnector};

use rustls::{Certificate, ClientConfig, PrivateKey, RootCertStore, ServerConfig};
use rustls_pemfile::{pkcs8_private_keys, rsa_private_keys};

use crate::error::Error;
use crate::settings::Shared;

/// Initialize our client [TlsConnector]. \
/// 1. Trust our own CA. ONLY our own CA.
/// 2. Set the client certificate and key
pub async fn get_tls_connector(settings: &Shared) -> Result<TlsConnector, Error> {
    // Only trust server-certificates signed with our own CA.
    let ca = load_ca(&settings.daemon_cert())?;
    let mut cert_store = RootCertStore::empty();
    cert_store.add(&ca).map_err(|err| {
        Error::CertificateFailure(format!("Failed to build RootCertStore: {}", err))
    })?;

    let config: ClientConfig = ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
        .expect("Couldn't enforce TLS1.2 and TLS 1.3. This is a bug.")
        .with_root_certificates(cert_store)
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(config)))
}

/// Configure the server using rusttls. \
/// A TLS server needs a certificate and a fitting private key.
pub fn get_tls_listener(settings: &Shared) -> Result<TlsAcceptor, Error> {
    // Set the server-side key and certificate that should be used for all communication.
    let certs = load_certs(&settings.daemon_cert())?;
    let key = load_key(&settings.daemon_key())?;

    let config = ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
        .expect("Couldn't enforce TLS1.2 and TLS 1.3. This is a bug.")
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| {
            Error::CertificateFailure(format!("Failed to build TLS Acceptor: {}", err))
        })?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

/// Load the passed certificates file
fn load_certs(path: &Path) -> Result<Vec<Certificate>, Error> {
    let file = File::open(path)
        .map_err(|_| Error::FileNotFound(format!("Cannot open cert {:?}", path)))?;
    let certs: Vec<Certificate> = rustls_pemfile::certs(&mut BufReader::new(file))
        .map_err(|_| Error::CertificateFailure("Failed to parse daemon certificate.".into()))?
        .into_iter()
        .map(Certificate)
        .collect();

    Ok(certs)
}

/// Load the passed keys file.
/// Only the first key will be used. It should match the certificate.
fn load_key(path: &Path) -> Result<PrivateKey, Error> {
    let file =
        File::open(path).map_err(|_| Error::FileNotFound(format!("Cannot open key {:?}", path)))?;

    // Try to read pkcs8 format first
    let keys = pkcs8_private_keys(&mut BufReader::new(&file))
        .map_err(|_| Error::CertificateFailure("Failed to parse pkcs8 format.".into()));

    if let Ok(keys) = keys {
        if let Some(key) = keys.into_iter().next() {
            return Ok(PrivateKey(key));
        }
    }

    // Try the normal rsa format afterwards.
    let keys = rsa_private_keys(&mut BufReader::new(file))
        .map_err(|_| Error::CertificateFailure("Failed to parse daemon key.".into()))?;

    if let Some(key) = keys.into_iter().next() {
        return Ok(PrivateKey(key));
    }

    Err(Error::CertificateFailure(format!(
        "Couldn't extract private key from keyfile {:?}",
        path
    )))
}

fn load_ca(path: &Path) -> Result<Certificate, Error> {
    let file = File::open(path)
        .map_err(|_| Error::FileNotFound(format!("Cannot open cert {:?}", path)))?;

    let cert = rustls_pemfile::certs(&mut BufReader::new(file))
        .map_err(|_| Error::CertificateFailure("Failed to parse daemon certificate.".into()))?
        .into_iter()
        .map(Certificate)
        .next()
        .ok_or_else(|| Error::CertificateFailure("Couldn't find CA certificate in file".into()))?;

    Ok(cert)
}
