//! Socket handling is platform specific code.
//!
//! The submodules of this module represent the different implementations for
//! each supported platform.
//! Depending on the target, the respective platform is read and loaded into this scope.

/// Shared socket logic
#[cfg_attr(not(target_os = "windows"), path = "unix.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod platform;
use async_trait::async_trait;

pub use self::platform::*;
use crate::error::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

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

pub trait Stream: AsyncRead + AsyncWrite + Unpin + Send {}
impl Stream for tokio_rustls::server::TlsStream<TcpStream> {}
impl Stream for tokio_rustls::client::TlsStream<TcpStream> {}
