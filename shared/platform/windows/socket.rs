use anyhow::Result;
use async_std::io::{Read, Write};
use async_std::net::{TcpListener, TcpStream};
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

impl GenericSocket for TcpStream {}

pub async fn get_listener(
    unix_socket_path: Option<String>,
    port: Option<String>,
) -> Result<Box<dyn GenericListener>> {
    let port = port.unwrap();
    let address = format!("127.0.0.1:{}", port);
    Ok(Box::new(TcpListener::bind(address).await?))
}
