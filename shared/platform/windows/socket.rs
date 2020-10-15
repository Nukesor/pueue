use anyhow::{Context, Result};
use async_std::io::{Read, Write};
use async_std::net::{TcpListener, TcpStream};
use async_trait::async_trait;

#[async_trait]
pub trait GenericListener: Sync + Send {
    async fn accept<'a>(&'a self) -> Result<Socket>;
}

#[async_trait]
impl GenericListener for TcpListener {
    async fn accept<'a>(&'a self) -> Result<Socket> {
        let (socket, _) = self.accept().await?;
        Ok(Box::new(socket))
    }
}

pub trait GenericSocket: Read + Write + Unpin + Send + Sync {}
impl GenericSocket for TcpStream {}

pub type Listener = Box<dyn GenericListener>;
pub type Socket = Box<dyn GenericSocket>;

pub async fn get_client(_unix_socket_path: Option<String>, port: Option<String>) -> Result<Socket> {
    // Don't allow anything else than loopback until we have proper crypto
    let address = format!("127.0.0.1:{}", port.unwrap());

    // Connect to socket
    let socket = TcpStream::connect(&address)
        .await
        .context("Failed to connect to the daemon. Did you start it?")?;

    Ok(Box::new(socket))
}

pub async fn get_listener(
    _unix_socket_path: Option<String>,
    port: Option<String>,
) -> Result<Listener> {
    let port = port.unwrap();
    let address = format!("127.0.0.1:{}", port);
    Ok(Box::new(TcpListener::bind(address).await?))
}
