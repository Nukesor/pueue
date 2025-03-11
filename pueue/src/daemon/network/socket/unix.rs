use std::{
    fs::{Permissions, set_permissions},
    os::unix::fs::PermissionsExt,
};

use pueue_lib::{Error, network::protocol::*, settings::Shared};
use tokio::net::{TcpListener, UnixSocket};

use crate::{
    daemon::network::tls::{TlsTcpListener, get_tls_listener},
    internal_prelude::*,
};

/// Unix specific cleanup handling when getting a SIGINT/SIGTERM.
pub fn socket_cleanup(settings: &Shared) -> Result<(), std::io::Error> {
    // Clean up the unix socket if we're using it and it exists.
    if settings.use_unix_socket && settings.unix_socket_path().exists() {
        std::fs::remove_file(settings.unix_socket_path())?;
    }

    Ok(())
}

/// Get a new listener for the daemon. \
/// This can either be a UnixListener or a TCPlistener, depending on the parameters.
pub async fn get_listener(settings: &Shared) -> Result<GenericListener, Error> {
    if settings.use_unix_socket {
        let socket_path = settings.unix_socket_path();
        info!("Using unix socket at: {socket_path:?}");

        // Check, if the socket already exists
        // In case it does, we have to check, if it's an active socket.
        // If it is, we have to throw an error, because another daemon is already running.
        // Otherwise, we can simply remove it.
        if socket_path.exists() {
            if get_client_stream(settings).await.is_ok() {
                return Err(Error::UnixSocketExists);
            }

            std::fs::remove_file(&socket_path).map_err(|err| {
                Error::IoPathError(socket_path.clone(), "removing old socket", err)
            })?;
        }

        // The various nix platforms handle socket permissions in different
        // ways, but generally prevent the socket's permissions from being
        // changed once it is being listened on.
        let socket = UnixSocket::new_stream()
            .map_err(|err| Error::IoError("creating unix socket".to_string(), err))?;
        socket.bind(&socket_path).map_err(|err| {
            Error::IoPathError(socket_path.clone(), "binding unix socket to path", err)
        })?;

        if let Some(mode) = settings.unix_socket_permissions {
            set_permissions(&socket_path, Permissions::from_mode(mode)).map_err(|err| {
                Error::IoPathError(
                    socket_path.clone(),
                    "setting permissions on unix socket",
                    err,
                )
            })?;
        }

        let unix_listener = socket.listen(1024).map_err(|err| {
            Error::IoPathError(socket_path.clone(), "listening on unix socket", err)
        })?;

        return Ok(Box::new(unix_listener));
    }

    // This is the listener, which accepts low-level TCP connections
    let address = format!("{}:{}", &settings.host, &settings.port);
    info!("Binding to address: {address}");
    let tcp_listener = TcpListener::bind(&address)
        .await
        .map_err(|err| Error::IoError("binding tcp listener to address".to_string(), err))?;

    // This is the TLS acceptor, which initializes the TLS layer
    let tls_acceptor = get_tls_listener(settings)?;

    // Create a struct, which accepts connections and initializes a TLS layer in one go.
    let tls_listener = TlsTcpListener {
        tcp_listener,
        tls_acceptor,
    };

    Ok(Box::new(tls_listener))
}
