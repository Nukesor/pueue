use byteorder::{BigEndian, WriteBytesExt};
use failure::Error;
use futures::Future;
use tokio::prelude::*;
use tokio_io::io as tokio_io;
use tokio_uds::UnixStream;

use crate::client::cli::handle_cli;
use crate::communication::local::get_socket_path;
use crate::communication::message::*;
use crate::settings::Settings;

/// The client
pub struct Client {
    settings: Settings,
    message: Message,
    response: Option<String>,
    communication_future: Option<Box<Future<Item = (UnixStream, Vec<u8>), Error = Error> + Send>>,
}

impl Client {
    pub fn new(settings: Settings) -> Self {
        let message = handle_cli();

        Client {
            settings: settings,
            message: message,
            response: None,
            communication_future: None,
        }
    }

    /// Send a message to the daemon.
    /// The JSON payload is highly dependent on the commandline input parameters
    /// Some payloads are serialized `Add` or `Remove` messages.
    /// Before we send the actual payload, a header is sent with two u64.
    /// One signals the type of the message, whilst the other signals the length of the payload.
    pub fn send_message(&mut self) {
        // Early return if we are already waiting for a future.
        if self.communication_future.is_some() {
            return;
        }

        // Get command
        let command_index = get_message_index(&self.message.message_type);

        // Prepare command for transfer and determine message byte length
        let byte_size = self.message.payload.chars().count() as u64;
        let payload = self.message.payload.clone();

        let mut header = vec![];
        header.write_u64::<BigEndian>(byte_size).unwrap();
        header.write_u64::<BigEndian>(command_index).unwrap();

        // Send the request size header first.
        // Afterwards send the request.
        let communication_future = UnixStream::connect(get_socket_path(&self.settings))
            .and_then(move |stream| tokio_io::write_all(stream, header))
            .and_then(move |(stream, _written)| tokio_io::write_all(stream, payload))
            .and_then(|(stream, _written)| tokio_io::read_to_end(stream, Vec::new()));

        self.communication_future = Some(Box::new(
            communication_future.map_err(|error| Error::from(error)),
        ));
    }

    /// Receive the response of the daemon and handle it.
    pub fn receive_answer(&mut self) -> bool {
        // Now receive the response until the connection closes.
        let result = self.communication_future.poll();

        // Handle socket error
        if result.is_err() {
            println!("Socket errored during read");
            println!("{:?}", result.err());

            panic!("Communication failed.");
        }

        // We received a response from the daemon. Handle it
        match result.unwrap() {
            Async::Ready(received_bytes_result) => {
                // Check whether we received something from the daemon.
                let (_, received_bytes) =
                    if let Some((stream, received_bytes)) = received_bytes_result {
                        (stream, received_bytes)
                    } else {
                        // Handle socket error
                        println!("Received an empty message from the daemon.");
                        panic!("Communication failed.");
                    };

                // Extract response and handle invalid utf8
                let response_result = String::from_utf8(received_bytes);

                let response = if let Ok(response) = response_result {
                    response
                } else {
                    println!("Didn't receive valid utf8.");
                    println!("{:?}", response_result.err());
                    panic!("Communication failed.");
                };

                self.response = Some(response);

                true
            }
            Async::NotReady => false,
        }
    }

    /// Handle the response of the daemon.
    pub fn handle_response(&self) -> bool {
        let response = if let Some(ref response) = self.response {
            response
        } else {
            return false;
        };

        println!("{}", &response);

        return true;
    }
}

impl Future for Client {
    type Item = ();
    type Error = Error;

    /// The poll function of the client.
    /// Send a message, receive the response and handle it accordingly to the current task.
    fn poll(&mut self) -> Result<Async<()>, Self::Error> {
        // Create the message payload and send it to the daemon.
        self.send_message();

        // Check if we can receive the response from the daemon
        let answer_received = self.receive_answer();

        // Return NotReady until the response has been received and handled.
        if answer_received {
            // Handle the response from the daemon
            self.handle_response();

            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}
