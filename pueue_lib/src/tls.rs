//! Helper functions for reading and handling TLS certificates.
use std::path::Path;

use rustls::pki_types::{CertificateDer, pem::PemObject};

use crate::error::Error;

/// Load a daemon's certificate from a given path.
///
/// This certificate needs to be provided when connecting via
/// [ConnectionSettings::TlsTcpSocket](crate::network::socket::ConnectionSettings::TlsTcpSocket)
///
/// If the pem file contains multiple certificates, the first one is picked.
pub fn load_certificate<'a>(path: &Path) -> Result<CertificateDer<'a>, Error> {
    CertificateDer::from_pem_file(path).map_err(|err| {
        Error::CertificateFailure(format!("Failed to parse daemon certificate:\n{err:?}"))
    })
}
