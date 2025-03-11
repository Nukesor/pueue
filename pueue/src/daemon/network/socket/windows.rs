use tokio::net::TcpListener;

use pueue_lib::{error::Error, network::socket::GenericListener, settings::Shared};

use crate::{
    daemon::network::tls::{TlsTcpListener, get_tls_listener},
    internal_prelude::*,
};

/// Windowsspecific cleanup handling when getting a SIGINT/SIGTERM.
pub fn socket_cleanup(_settings: &Shared) -> Result<(), Error> {
    Ok(())
}

/// Get a new [TlsTcpListener] for the daemon.
pub async fn get_listener(settings: &Shared) -> Result<GenericListener, Error> {
    // This is the listener, which accepts low-level TCP connections
    let address = format!("{}:{}", settings.host, settings.port);
    let tcp_listener = TcpListener::bind(&address).await.map_err(|err| {
        Error::Connection(format!("Failed to listen on address {address}. {err}"))
    })?;

    // This is the TLS acceptor, which initializes the TLS layer
    let tls_acceptor = get_tls_listener(settings)?;

    // Create a struct, which accepts connections and initializes a TLS layer in one go.
    let tls_listener = TlsTcpListener {
        tcp_listener,
        tls_acceptor,
    };

    Ok(Box::new(tls_listener))
}
