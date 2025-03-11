//! Helper functions for reading and handling TLS certificates.
use std::{fs::File, io::BufReader, path::Path};

use rustls::pki_types::CertificateDer;

use crate::error::Error;

/// Load a daemon's certificate from a given path.
///
/// This certificate needs to be provided when connecting via
/// [ConnectionSettings::TlsTcpSocket](crate::network::socket::ConnectionSettings::TlsTcpSocket)
pub fn load_ca<'a>(path: &Path) -> Result<CertificateDer<'a>, Error> {
    let file = File::open(path)
        .map_err(|err| Error::IoPathError(path.to_path_buf(), "opening cert", err))?;

    let cert = rustls_pemfile::certs(&mut BufReader::new(file))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .map_err(|_| Error::CertificateFailure("Failed to parse daemon certificate.".into()))?
        .into_iter()
        .next()
        .ok_or_else(|| Error::CertificateFailure("Couldn't find CA certificate in file".into()))?;

    Ok(cert)
}
