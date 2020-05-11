use ::anyhow::{Context, Result};
use ::async_std::net::TcpStream;
use ::log::error;
use ::std::io::{self, Write};

use crate::cli::Opt;
use crate::edit::*;
use crate::output::*;
use ::pueue::message::*;
use ::pueue::protocol::*;
use ::pueue::settings::Settings;

/// Representation of a client.
/// For convenience purposes this logic has been wrapped in a struct.
/// The client is responsible for connecting to the daemon, sending an instruction
/// and interpreting the response.
///
/// Most commands are a simple ping-pong. Though, some commands require a more complex
/// communication pattern (e.g. `show -f`, which contiuously streams the output of a task).
pub struct Client {
    opt: Opt,
    daemon_address: String,
    message: Message,
    settings: Settings,
}

impl Client {
    pub fn new(settings: Settings, message: Message, opt: Opt) -> Result<Self> {
        // // Commandline argument overwrites the configuration files values for address
        // let address = if let Some(address) = opt.address.clone() {
        //     address
        // } else {
        //     settings.client.daemon_address
        // };

        // Commandline argument overwrites the configuration files values for port
        let port = if let Some(port) = opt.port.clone() {
            port
        } else {
            settings.client.daemon_port.clone()
        };

        // Don't allow anything else than loopback until we have proper crypto
        // let address = format!("{}:{}", address, port);
        let address = format!("127.0.0.1:{}", port);

        Ok(Client {
            opt,
            daemon_address: address,
            message,
            settings,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Connect to socket
        let mut socket = TcpStream::connect(&self.daemon_address)
            .await
            .context("Failed to connect to the daemon. Did you start it?")?;

        let secret = self.settings.client.secret.clone().into_bytes();
        send_bytes(secret, &mut socket).await?;

        // Create the message payload and send it to the daemon.
        send_message(self.message.clone(), &mut socket).await?;

        // Check if we can receive the response from the daemon
        let mut message = receive_message(&mut socket).await?;

        while self.handle_message(message, &mut socket).await? {
            // Check if we can receive the response from the daemon
            message = receive_message(&mut socket).await?;
        }

        Ok(())
    }

    /// Most returned messages can be handled in a generic fashion.
    /// However, some commands need some ping-pong or require continuous receiving of messages.
    ///
    /// If this function returns `Ok(true)`, the parent function will continue to receive
    /// and handle messages from the daemon. Otherwise the client will simply exit.
    async fn handle_message(&mut self, message: Message, socket: &mut TcpStream) -> Result<bool> {
        match message {
            Message::Success(text) => print_success(text),
            Message::Failure(text) => print_error(text),
            Message::StatusResponse(state) => print_state(state, &self.opt.cmd),
            Message::LogResponse(task_logs) => print_logs(task_logs, &self.opt.cmd, &self.settings),
            Message::EditResponse(message) => {
                // Create a new message with the edited command
                let message = edit(message, &self.opt.cmd);
                send_message(message, socket).await?;
                return Ok(true);
            }
            Message::Stream(text) => {
                print!("{}", text);
                io::stdout().flush().unwrap();
                return Ok(true);
            }
            _ => error!("Received unhandled response message"),
        };

        Ok(false)
    }
}
