use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Error, Result};
use async_tls::{TlsAcceptor, TlsConnector};
use rustls::{
    internal::pemfile::{certs, pkcs8_private_keys, rsa_private_keys},
    NoClientAuth,
};
use rustls::{Certificate, ClientConfig, PrivateKey, ServerConfig};

use crate::settings::Shared;

/// Initialize our client [TlsConnector]. \
/// 1. Trust our own CA. ONLY our own CA.
/// 2. Set the client certificate and key
pub async fn get_tls_connector(settings: &Shared) -> Result<TlsConnector> {
    let mut config = ClientConfig::new();

    // Trust server-certificates signed with our own CA.
    let mut ca = load_ca(&settings.daemon_cert())?;
    config
        .root_store
        .add_pem_file(&mut ca)
        .map_err(|_| anyhow!("Failed to add CA to client root store."))?;

    Ok(TlsConnector::from(Arc::new(config)))
}

/// Configure the server using rusttls. \
/// A TLS server needs a certificate and a fitting private key.
pub fn get_tls_listener(settings: &Shared) -> Result<TlsAcceptor> {
    let mut config = ServerConfig::new(NoClientAuth::new());

    // Set the mtu to 1500, since we might have non-local communication.
    config.mtu = Some(1500);

    // Set the server-side key and certificate that should be used for any communication
    let certs = load_certs(&settings.daemon_cert())?;
    let mut keys = load_keys(&settings.daemon_key())?;
    if keys.is_empty() {
        bail!(
            "Couldn't extract private key from keyfile {:?}",
            &settings.daemon_key()
        );
    }

    config
        // set this server to use one cert together with the loaded private key
        .set_single_cert(certs, keys.remove(0))
        .map_err(Error::new)
        .context("Failed to set single certificate for daemon.")?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

/// Load the passed certificates file
fn load_certs(path: &Path) -> Result<Vec<Certificate>> {
    let file = File::open(path).context(format!("Cannot open cert {:?}", path))?;
    certs(&mut BufReader::new(file)).map_err(|_| anyhow!("Failed to parse daemon certificate."))
}

/// Load the passed keys file
fn load_keys(path: &Path) -> Result<Vec<PrivateKey>> {
    let file = File::open(path).context(format!("Cannot open key {:?}", path))?;
    // Try to read pkcs8 format first
    let keys = pkcs8_private_keys(&mut BufReader::new(&file))
        .map_err(|_| anyhow!("Failed to parse daemon key."))?;

    if !keys.is_empty() {
        return Ok(keys);
    }

    // Try the normal rsa format afterwards.
    rsa_private_keys(&mut BufReader::new(file)).map_err(|_| anyhow!("Failed to parse daemon key."))
}

fn load_ca(path: &Path) -> Result<Cursor<Vec<u8>>> {
    let file = std::fs::read(path).map_err(|_| anyhow!("Failed to read CA file."))?;
    Ok(Cursor::new(file))
}
