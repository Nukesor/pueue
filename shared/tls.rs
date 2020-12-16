use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::Arc;

use async_std::io::{Error, ErrorKind, Result};
use async_tls::TlsConnector;
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{Certificate, ClientConfig, NoClientAuth, PrivateKey, ServerConfig};

use crate::settings::Settings;

pub async fn get_client_tls_connector(settings: &Settings) -> Result<TlsConnector> {
    let mut config = ClientConfig::new();
    let file = async_std::fs::read(&settings.shared.client_cert).await?;
    let mut pem = Cursor::new(file);

    config
        .root_store
        .add_pem_file(&mut pem)
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "invalid cert"))?;

    Ok(TlsConnector::from(Arc::new(config)))
}

/// Configure the server using rusttls
/// See https://docs.rs/rustls/0.16.0/rustls/struct.ServerConfig.html for details
///
/// A TLS server needs a certificate and a fitting private key
pub fn load_config(settings: &Settings) -> Result<ServerConfig> {
    let certs = load_certs(&settings.shared.daemon_cert)?;
    let mut keys = load_keys(&settings.shared.daemon_key)?;

    // we don't use client authentication
    let mut config = ServerConfig::new(NoClientAuth::new());
    config
        // set this server to use one cert together with the loaded private key
        .set_single_cert(certs, keys.remove(0))
        .map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;

    Ok(config)
}

/// Load the passed certificates file
fn load_certs(path: &Path) -> Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "invalid cert"))
}

/// Load the passed keys file
fn load_keys(path: &Path) -> Result<Vec<PrivateKey>> {
    rsa_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "invalid key"))
}
