use anyhow::{Context, Result};
use async_std::io::{Read, Write};
use async_std::net::{TcpListener, TcpStream};
use async_trait::async_trait;

#[async_trait]
pub trait GenericListener: Sync + Send {
    async fn accept<'a>(&'a self) -> Result<GenericStream>;
}

#[async_trait]
impl GenericListener for TcpListener {
    async fn accept<'a>(&'a self) -> Result<GenericStream> {
        let (stream, _) = self.accept().await?;
        Ok(Box::new(stream))
    }
}

pub trait Stream: Read + Write + Unpin + Send + Sync {}
impl Stream for TcpStream {}

pub type Listener = Box<dyn GenericListener>;
pub type GenericStream = Box<dyn Stream>;

pub async fn get_client(_unix_socket_path: Option<String>, port: Option<String>) -> Result<GenericStream> {
    // Don't allow anything else than loopback until we have proper crypto
    let address = format!("127.0.0.1:{}", port.unwrap());

    // Connect to socket
    let stream = TcpStream::connect(&address)
        .await
        .context("Failed to connect to the daemon. Did you start it?")?;

    Ok(Box::new(stream))
}

pub async fn get_listener(
    _unix_socket_path: Option<String>,
    port: Option<String>,
) -> Result<Listener> {
    let port = port.unwrap();
    let address = format!("127.0.0.1:{}", port);
    Ok(Box::new(TcpListener::bind(address).await?))
}
