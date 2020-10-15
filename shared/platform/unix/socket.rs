use anyhow::{Context, Result};
use async_std::io::{Read, Write};
use async_std::net::{TcpListener, TcpStream};
use async_std::os::unix::net::{UnixListener, UnixStream};
use async_trait::async_trait;

pub trait GenericSocket: Read + Write + Unpin + Send + Sync {}
pub type SocketBox = Box<dyn GenericSocket>;

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

impl GenericSocket for TcpStream {}
impl GenericSocket for UnixStream {}

pub async fn get_client(
    unix_socket_path: Option<String>,
    port: Option<String>,
) -> Result<Box<dyn GenericSocket>> {
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

pub async fn get_listener(
    unix_socket_path: Option<String>,
    port: Option<String>,
) -> Result<Box<dyn GenericListener>> {
    if let Some(socket_path) = unix_socket_path {
        return Ok(Box::new(UnixListener::bind(socket_path).await?));
    }

    let port = port.unwrap();
    let address = format!("127.0.0.1:{}", port);
    Ok(Box::new(TcpListener::bind(address).await?))
}
