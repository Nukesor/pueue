//! Socket handling is platform specific code.
//!
//! The submodules of this module represent the different implementations for
//! each supported platform.
//! Depending on the target, the respective platform is read and loaded into this scope.

#[cfg(not(target_os = "windows"))]
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use rustls::{ClientConfig, RootCertStore, pki_types::CertificateDer};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;

use crate::error::Error;
#[cfg(feature = "settings")]
use crate::{settings::Shared, tls::load_certificate};

/// Shared socket logic
#[cfg_attr(not(target_os = "windows"), path = "unix.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod platform;
pub use platform::*;

/// A new trait, which can be used to represent Unix- and TcpListeners. \
/// This is necessary to easily write generic functions where both types can be used.
#[async_trait]
pub trait Listener: Sync + Send {
    async fn accept<'a>(&'a self) -> Result<GenericStream, Error>;
}

/// Convenience type, so we don't have type write `Box<dyn Listener>` all the time.
pub type GenericListener = Box<dyn Listener>;
/// Convenience type, so we don't have type write `Box<dyn Stream>` all the time. \
/// This also prevents name collisions, since `Stream` is imported in many preludes.
pub type GenericStream = Box<dyn Stream>;

/// Describe how a client should connect to the daemon.
pub enum ConnectionSettings<'a> {
    #[cfg(not(target_os = "windows"))]
    UnixSocket { path: PathBuf },
    TlsTcpSocket {
        host: String,
        port: String,
        certificate: CertificateDer<'a>,
    },
}

/// Convenience conversion from [Shared] to [ConnectionSettings].
#[cfg(feature = "settings")]
impl TryFrom<Shared> for ConnectionSettings<'_> {
    type Error = crate::error::Error;

    fn try_from(value: Shared) -> Result<Self, Self::Error> {
        // Unix socket handling
        #[cfg(not(target_os = "windows"))]
        {
            if value.use_unix_socket {
                return Ok(ConnectionSettings::UnixSocket {
                    path: value.unix_socket_path(),
                });
            }
        }

        let cert = load_certificate(&value.daemon_cert())?;
        Ok(ConnectionSettings::TlsTcpSocket {
            host: value.host,
            port: value.port,
            certificate: cert,
        })
    }
}

pub trait Stream: AsyncRead + AsyncWrite + Unpin + Send {}
impl Stream for tokio_rustls::server::TlsStream<TcpStream> {}
impl Stream for tokio_rustls::client::TlsStream<TcpStream> {}

/// Initialize our client [TlsConnector]. \
/// 1. Trust our own CA. ONLY our own CA.
/// 2. Set the client certificate and key
pub async fn get_tls_connector(cert: CertificateDer<'_>) -> Result<TlsConnector, Error> {
    // Only trust server-certificates signed with our own CA.
    let mut cert_store = RootCertStore::empty();
    cert_store.add(cert).map_err(|err| {
        Error::CertificateFailure(format!("Failed to build RootCertStore: {err}"))
    })?;

    let config: ClientConfig = ClientConfig::builder()
        .with_root_certificates(cert_store)
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(config)))
}
