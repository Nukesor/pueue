use ::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ::failure::Error;
use ::futures::Future;
use ::std::io::Cursor;
use ::tokio::io as tokio_io;
use ::tokio::net::UnixStream;
use ::tokio::prelude::*;

use crate::client::cli::handle_cli;
use crate::communication::local::get_socket_path;
use crate::communication::message::*;
use crate::settings::Settings;

/// The client
pub struct Client {
    settings: Settings,
    message: Message,
    response: Option<String>,
    communication_future:
        Option<Box<dyn Future<Item = (UnixStream, Vec<u8>), Error = Error> + Send>>,
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

        // Prepare command for transfer and determine message byte size
        let payload = serde_json::to_string(&self.message)
            .expect("Failed to serialize message.")
            .into_bytes();
        let byte_size = payload.len() as u64;

        let mut header = vec![];
        header.write_u64::<BigEndian>(byte_size).unwrap();

        // Send the request size header first.
        // Afterwards send the request.
        let communication_future = UnixStream::connect(get_socket_path(&self.settings))
            .and_then(|stream| tokio_io::write_all(stream, header))
            .and_then(|(stream, _written)| tokio_io::write_all(stream, payload))
            .and_then(|(stream, _written)| tokio_io::read_exact(stream, vec![0; 8]))
            .and_then(|(stream, header)| {
                // Extract the instruction size from the header bytes
                let mut header = Cursor::new(header);
                let instruction_size = header.read_u64::<BigEndian>().unwrap() as usize;

                tokio_io::read_exact(stream, vec![0; instruction_size])
            })
            .map_err(|error| Error::from(error));

        self.communication_future = Some(Box::new(communication_future));
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
    type Error = ();

    /// The poll function of the client.
    /// Send a message, receive the response and handle it accordingly to the current task.
    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
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
