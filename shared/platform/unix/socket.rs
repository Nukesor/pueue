use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use async_std::io::{Read, Write};
use async_std::net::{TcpListener, TcpStream};
use async_std::os::unix::net::{UnixListener, UnixStream};
use async_trait::async_trait;

#[async_trait]
pub trait GenericListener: Sync + Send {
    async fn accept<'a>(&'a self) -> Result<Box<dyn GenericSocket>>;
}

#[async_trait]
impl GenericListener for TcpListener {
    async fn accept<'a>(&'a self) -> Result<Box<dyn GenericSocket>> {
        let (socket, _) = self.accept().await?;
        Ok(Box::new(socket))
    }
}

#[async_trait]
impl GenericListener for UnixListener {
    async fn accept<'a>(&'a self) -> Result<Box<dyn GenericSocket>> {
        let (socket, _) = self.accept().await?;
        Ok(Box::new(socket))
    }
}

pub trait GenericSocket: Read + Write + Unpin + Send + Sync {}
impl GenericSocket for TcpStream {}
impl GenericSocket for UnixStream {}

pub type Listener = Box<dyn GenericListener>;
pub type Socket = Box<dyn GenericSocket>;

/// Get a new stream for the client.
/// This can either be a UnixStream or a TCPStream,
/// which depends on the parameters.
pub async fn get_client(unix_socket_path: Option<String>, port: Option<String>) -> Result<Socket> {
    if let Some(socket_path) = unix_socket_path {
        let stream = UnixStream::connect(socket_path).await?;
        return Ok(Box::new(stream));
    }

    // Don't allow anything else than loopback until we have proper crypto
    // let address = format!("{}:{}", address, port);
    let address = format!("127.0.0.1:{}", port.unwrap());

    // Connect to socket
    let socket = TcpStream::connect(&address)
        .await
        .context("Failed to connect to the daemon. Did you start it?")?;

    Ok(Box::new(socket))
}

/// Get a new listener for the daemon.
/// This can either be a UnixListener or a TCPlistener,
/// which depends on the parameters.
pub async fn get_listener(
    unix_socket_path: Option<String>,
    port: Option<String>,
) -> Result<Listener> {
    if let Some(socket_path) = unix_socket_path {
        // Check, if the socket already exists
        // In case it does, we have to check, if it's an active socket.
        // If it is, we have to throw an error, because another daemon is already running.
        // Otherwise, we can simply remove it.
        if PathBuf::from(&socket_path).exists() {
            if get_client(Some(socket_path.clone()), None).await.is_ok() {
                bail!(
                    "There seems to be an active pueue daemon.\n\
                      If you're sure there isn't, please remove the socket by hand \
                      inside the pueue_directory."
                );
            }

            std::fs::remove_file(&socket_path)?;
        }

        return Ok(Box::new(UnixListener::bind(socket_path).await?));
    }

    let port = port.unwrap();
    let address = format!("127.0.0.1:{}", port);
    Ok(Box::new(TcpListener::bind(address).await?))
}
