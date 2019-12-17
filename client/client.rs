use ::anyhow::Result;
use ::async_std::net::TcpStream;
use ::log::error;
use ::std::io::{self, Write};

use crate::cli::{Opt, SubCommand};
use crate::instructions::*;
use crate::output::*;
use ::pueue::message::*;
use ::pueue::protocol::*;
use ::pueue::settings::Settings;

/// Representation of a client
/// For convenience purposes this logic has been wrapped in a struct
/// The client is responsible for connecting to the daemon, sending an instruction
/// and interpreting the response
///
/// Most commands are a simple ping-pong. Though, some commands require a more complex
/// communication pattern (e.g. `show -f`, which contiuously streams the output of a task)
pub struct Client {
    opt: Opt,
    daemon_address: String,
    message: Message,
    secret: String,
}

impl Client {
    pub fn new(settings: Settings, message: Message, opt: Opt) -> Result<Self> {
        //        // Commandline argument overwrites the configuration files values for address
        //        let address = if let Some(address) = opt.address.clone() {
        //            address
        //        } else {
        //            settings.client.daemon_address
        //        };

        // Commandline argument overwrites the configuration files values for port
        let port = if let Some(port) = opt.port.clone() {
            port
        } else {
            settings.client.daemon_port
        };

        //        let address = format!("{}:{}", address, port);
        let address = format!("127.0.0.1:{}", port);

        Ok(Client {
            opt: opt,
            daemon_address: address,
            message: message,
            secret: settings.client.secret,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Connect to socket
        let mut socket = TcpStream::connect(&self.daemon_address).await?;

        let secret = self.secret.clone().into_bytes();
        send_bytes(secret, &mut socket).await?;

        // Create the message payload and send it to the daemon.
        send_message(&self.message, &mut socket).await?;

        // Check if we can receive the response from the daemon
        let mut message = receive_message(&mut socket).await?;

        while self.handle_message(message, &mut socket).await? {
            // Check if we can receive the response from the daemon
            message = receive_message(&mut socket).await?;
        }

        Ok(())
    }

    async fn handle_message(&mut self, message: Message, socket: &mut TcpStream) -> Result<bool> {
        // Handle some messages directly
        match message {
            Message::Success(text) => print_success(text),
            Message::Failure(text) => print_error(text),
            Message::Stream(text) => {
                print!("{}", text);
                io::stdout().flush().unwrap();
                return Ok(true);
            }
            _ => {
                // Other messages will be handled depending on the original cli-command
                match &self.opt.cmd {
                    SubCommand::Status { json } => print_state(message, *json),
                    SubCommand::Log { task_ids, json } => {
                        print_logs(message, task_ids.clone(), *json)
                    }
                    SubCommand::Edit { task_id: _ } => {
                        // Create a new message with the edited command
                        let message = edit(message);
                        send_message(&message, socket).await?;
                        return Ok(true);
                    }
                    _ => error!("Received unhandled response message"),
                }
            }
        };

        Ok(false)
    }
}
