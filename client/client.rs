use ::anyhow::Result;
use ::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ::log::error;
use ::std::io::Cursor;
use ::tokio::net::TcpStream;
use ::tokio::prelude::*;

use crate::cli::{Opt, SubCommand};
use crate::output::*;
use ::pueue::message::*;
use ::pueue::settings::Settings;

/// The client
pub struct Client {
    opt: Opt,
    settings: Settings,
    message: Message,
}

impl Client {
    pub fn new(settings: Settings, message: Message, opt: Opt) -> Result<Self> {
        Ok(Client {
            opt: opt,
            settings: settings,
            message: message,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Connect to stream
        let address = format!(
            "{}:{}",
            self.settings.daemon.address, self.settings.daemon.port
        );
        let mut stream = TcpStream::connect(address).await?;

        // Create the message payload and send it to the daemon.
        self.send_message(&mut stream).await?;

        // Check if we can receive the response from the daemon
        let response = self.receive_answer(&mut stream).await?;
        // Interpret the response
        let message: Message = serde_json::from_str(&response)?;

        // Handle some messages directly
        match message {
            Message::Success(text) => print_success(text),
            Message::Failure(text) => print_error(text),
            _ => {
                // Other messages will be handled depending on the original cli-command
                match &self.opt.cmd {
                    SubCommand::Status => print_state(message),
                    SubCommand::Log { task_ids } => print_logs(message, task_ids.clone()),
                    _ => error!("Received unhandled response message"),
                }
            }
        }

        Ok(())
    }

    /// Send a message to the daemon.
    /// The JSON payload is highly dependent on the commandline input parameters
    /// Some payloads are serialized `Add` or `Remove` messages.
    /// Before we send the actual payload, a header is sent with two u64.
    /// The first represents the type of the message, the second is length of the payload.
    async fn send_message(&mut self, stream: &mut TcpStream) -> Result<()> {
        // Prepare command for transfer and determine message byte size
        let payload = serde_json::to_string(&self.message)
            .expect("Failed to serialize message.")
            .into_bytes();
        let byte_size = payload.len() as u64;

        let mut header = vec![];
        header.write_u64::<BigEndian>(byte_size).unwrap();

        // Send the request size header first.
        // Afterwards send the request.
        stream.write_all(&header).await?;
        stream.write_all(&payload).await?;

        Ok(())
    }

    /// Receive the response of the daemon and handle it.
    async fn receive_answer(&mut self, stream: &mut TcpStream) -> Result<String> {
        // Extract the instruction size from the header bytes
        let mut header_buffer = vec![0; 8];
        stream.read(&mut header_buffer).await?;
        let mut header = Cursor::new(header_buffer);
        let instruction_size = header.read_u64::<BigEndian>().unwrap() as usize;

        // Receive the instruction
        let mut buffer = vec![0; instruction_size];
        stream.read(&mut buffer).await?;

        Ok(String::from_utf8(buffer)?)
    }
}
