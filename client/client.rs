use ::anyhow::Result;
use ::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ::log::error;
use ::std::io::Cursor;
use ::tokio::net::TcpStream;
use ::tokio::prelude::*;

use crate::cli::{Opt, SubCommand};
use crate::output::*;
use crate::instructions::*;
use ::pueue::message::*;
use ::pueue::settings::Settings;

/// The client
pub struct Client {
    opt: Opt,
    daemon_address: String,
    message: Message,
}

impl Client {
    pub fn new(settings: Settings, message: Message, opt: Opt) -> Result<Self> {
        let address = format!(
            "{}:{}",
            settings.daemon.address, settings.daemon.port
        );

        Ok(Client {
            opt: opt,
            daemon_address: address,
            message: message,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Connect to stream
        let mut stream = TcpStream::connect(&self.daemon_address).await?;

        // Create the message payload and send it to the daemon.
        send_message(&self.message, &mut stream).await?;

        // Check if we can receive the response from the daemon
        let message = receive_answer(&mut stream).await?;

        self.handle_message(message).await?;

        Ok(())
    }

    async fn handle_message(&mut self, message: Message) -> Result<()> {
        // Handle some messages directly
        match message {
            Message::Success(text) => print_success(text),
            Message::Failure(text) => print_error(text),
            _ => {
                // Other messages will be handled depending on the original cli-command
                match &self.opt.cmd {
                    SubCommand::Status => print_state(message),
                    SubCommand::Log { task_ids } => print_logs(message, task_ids.clone()),
                    SubCommand::Edit { task_id: _ } => {
                        // Create a new message with the edited command
                        let message = edit(message);
                        // Open a new connection (One connection is only viable for a single request)
                        let mut stream = TcpStream::connect(&self.daemon_address).await?;
                        send_message(&message, &mut stream).await?;
                        let message = receive_answer(&mut stream).await?;
                        match message {
                             Message::Success(text) => print_success(text),
                             _ => error!("Got involid response {:?}", message)
                        }
                    }
                    _ => error!("Received unhandled response message"),
                }
            }
        };

        Ok(())
    }

}

/// Send a message to the daemon.
/// The JSON payload is highly dependent on the commandline input parameters
/// Some payloads are serialized `Add` or `Remove` messages.
/// Before we send the actual payload, a header is sent with two u64.
/// The first represents the type of the message, the second is length of the payload.
async fn send_message(message: &Message,  stream: &mut TcpStream) -> Result<()> {
    // Prepare command for transfer and determine message byte size
    let payload = serde_json::to_string(message)
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
async fn receive_answer(stream: &mut TcpStream) -> Result<Message> {
    // Extract the instruction size from the header bytes
    let mut header_buffer = vec![0; 8];
    stream.read(&mut header_buffer).await?;
    let mut header = Cursor::new(header_buffer);
    let instruction_size = header.read_u64::<BigEndian>().unwrap() as usize;

    // Receive the instruction
    let mut buffer = vec![0; instruction_size];
    stream.read(&mut buffer).await?;

    let payload = String::from_utf8(buffer)?;
    // Interpret the response
    let message: Message = serde_json::from_str(&payload)?;

    Ok(message)
}
