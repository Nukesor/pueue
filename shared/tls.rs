use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use rustls::{
    internal::pemfile::certs, internal::pemfile::rsa_private_keys, Certificate, NoClientAuth,
    PrivateKey, ServerConfig,
};

use crate::settings::Settings;

/// Configure the server using rusttls
/// See https://docs.rs/rustls/0.16.0/rustls/struct.ServerConfig.html for details
///
/// A TLS server needs a certificate and a fitting private key
pub fn load_config(settings: &Settings) -> io::Result<ServerConfig> {
    let certs = load_certs(&settings.shared.daemon_cert)?;
    let mut keys = load_keys(&settings.shared.daemon_key)?;

    // we don't use client authentication
    let mut config = ServerConfig::new(NoClientAuth::new());
    config
        // set this server to use one cert together with the loaded private key
        .set_single_cert(certs, keys.remove(0))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    Ok(config)
}

/// Load the passed certificates file
fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
}

/// Load the passed keys file
fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
    rsa_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
}
