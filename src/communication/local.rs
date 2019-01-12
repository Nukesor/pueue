use ::failure::Error;
use ::futures::{Future, Poll};
use ::std::fs::remove_file;
use ::std::path::Path;
use ::tokio::net::{UnixListener, UnixStream};
use ::tokio::prelude::*;

use crate::settings::Settings;

/// Create a new unix listener.
/// In case a socket already exists it will be removed
pub fn get_unix_listener(settings: &Settings) -> UnixListener {
    let socket_path = get_socket_path(&settings);

    // Remove old socket
    if Path::new(&socket_path).exists() {
        remove_file(&socket_path).expect("Failed to remove old socket.");
        println!("Remove old socket.");
    }

    println!("Creating socket at {}", socket_path);

    UnixListener::bind(socket_path).expect("Failed to create unix socket.")
}

/// Helper function to create the socket path used by clients and the daemon.
/// Panic in case we can't create the socket path, since this is a critical error.
pub fn get_socket_path(settings: &Settings) -> String {
    let path = Path::new(settings.common.local_socket_dir.as_str())
        .join(format!("pueue_{}.sock", settings.common.group_id));

    path.as_path()
        .to_str()
        .expect("Unable to create socket path.")
        .to_string()
}
