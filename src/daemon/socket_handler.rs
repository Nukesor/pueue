use ::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ::failure::Error;
use ::futures::prelude::*;
use ::futures::Future;
use ::std::collections::HashMap;
use ::std::io::Cursor;
use ::std::net::Shutdown;
use ::tokio::io as tokio_io;
use ::tokio::net::{UnixListener, UnixStream};
use ::uuid::Uuid;

use crate::communication::local::*;
use crate::communication::message::*;
use crate::settings::Settings;

pub struct SocketHandler {
    unix_listener: UnixListener,
    unix_incoming: Vec<Box<dyn Future<Item = (UnixStream, Vec<u8>), Error = Error> + Send>>,
    unix_responses:
        HashMap<Uuid, Box<dyn Future<Item = (UnixStream, Vec<u8>), Error = Error> + Send>>,
    unix_sockets: HashMap<Uuid, UnixStream>,
    unix_response_queue: Vec<Message>,
}

impl SocketHandler {
    /// Create a new daemon.
    /// This function also handle the creation of other components,
    /// such as the queue, sockets and the process handler.
    pub fn new(settings: &Settings) -> Self {
        SocketHandler {
            unix_listener: get_unix_listener(&settings),
            unix_incoming: Vec::new(),
            unix_responses: HashMap::new(),
            unix_sockets: HashMap::new(),
            unix_response_queue: Vec::new(),
        }
    }

    /// Poll the unix listener and accept new incoming connections
    /// Create a new future for receiving the instruction and add it to unix_incoming
    pub fn accept_incoming(&mut self) {
        loop {
            // Poll if we have a new incoming connection.
            // In case we don't, break the loop
            let accept_result = self.unix_listener.poll_accept();
            let accept_future = if let Ok(future) = accept_result {
                future
            } else {
                println!("Failed to accept incoming unix connection.");
                println!("{:?}", accept_result.err());
                continue;
            };

            // Check if we can accept an incoming connection or if we need to wait a little.
            match accept_future {
                Async::Ready((stream, _socket_addr)) => {
                    // First read the header to determine the size of the instruction
                    let incoming = tokio_io::read_exact(stream, vec![0; 8])
                        .and_then(|(stream, header)| {
                            // Extract the instruction size from the header bytes
                            let mut header = Cursor::new(header);
                            let instruction_size = header.read_u64::<BigEndian>().unwrap() as usize;

                            tokio_io::read_exact(stream, vec![0; instruction_size])
                        })
                        .map_err(|error| Error::from(error));
                    self.unix_incoming.push(Box::new(incoming));
                }
                Async::NotReady => break,
            }
        }
    }

    /// Continuously poll the existing incoming futures.
    /// In case we received an instruction, handle it and create a response future.
    /// The response future is added to unix_responses and handled in a separate function.
    pub fn handle_incoming(&mut self) -> Vec<(Uuid, String)> {
        let mut instructions = Vec::new();
        let len = self.unix_incoming.len();

        for i in (0..len).rev() {
            let result = self.unix_incoming[i].poll();

            // Handle socket error
            if result.is_err() {
                println!("Socket errored during read");
                println!("{:?}", result.err());
                self.unix_incoming.remove(i);

                continue;
            }

            // We received an instruction from a client. Handle it
            match result.unwrap() {
                Async::Ready((stream, instruction_bytes)) => {
                    let instruction =
                        String::from_utf8(instruction_bytes).expect("Failed to create utf8 string");

                    println!("{}", instruction);
                    let hash_uuid = Uuid::new_v4();
                    self.unix_sockets.insert(hash_uuid, stream);
                    self.unix_incoming.remove(i);

                    instructions.push((hash_uuid, instruction));
                }
                Async::NotReady => {}
            }
        }

        instructions
    }

    /// Send or queue a vector of messages
    pub fn process_responses(&mut self, mut responses: Vec<(Uuid, Message)>) {
        while let Some((uuid, message)) = responses.pop() {
            self.send_or_queue_message(uuid, message);
        }
    }

    /// Send a message to a specific unix socket
    /// The uuid of the socket is contained inside the Message
    pub fn send_or_queue_message(&mut self, uuid: Uuid, message: Message) {
        if self.can_be_responded_to(&uuid) {
            self.send_message(uuid, message)
        } else if self.is_sending(&uuid) {
            self.unix_response_queue.push(message)
        } else {
            println!("Cannot send message. The unix socket doesn't seem to exist any longer.");
            if let Ok(message) = serde_json::to_string_pretty(&message) {
                println!("{}", message);
            }
        }
    }

    /// Create the response future for this message.
    fn send_message(&mut self, uuid: Uuid, message: Message) {
        let stream = self
            .unix_sockets
            .remove(&uuid)
            .expect("Tried to remove non-existing unix socket.");

        println!("Sending message");

        if let Ok(payload) = serde_json::to_string(&message) {
            let payload = payload.into_bytes();
            let byte_size = payload.len() as u64;

            let mut header = vec![];
            header.write_u64::<BigEndian>(byte_size).unwrap();

            let response_future = tokio_io::write_all(stream, header)
                .and_then(|(stream, _written)| {
                    println!("header sent");
                    tokio_io::write_all(stream, payload)
                })
                .map_err(|error| Error::from(error));
            self.unix_responses.insert(uuid, Box::new(response_future));
        } else {
            // TODO: proper error handling
            println!("Error creating message");
        }
    }

    /// Check whether a socket is available for sending something.
    fn can_be_responded_to(&mut self, uuid: &Uuid) -> bool {
        self.unix_sockets.contains_key(uuid)
    }

    /// Check whether the socket is already used for sending a response right now.
    fn is_sending(&mut self, uuid: &Uuid) -> bool {
        self.unix_responses.contains_key(uuid)
    }

    /// Check messages have been sent to the client.
    /// If a message has been successfully sent, add it unix_sockets again for further messages.
    pub fn check_responses(&mut self) {
        let mut not_ready = HashMap::new();
        for (uuid, mut future) in self.unix_responses.drain().take(1) {
            let result = future.poll();

            // Handle socket error
            if result.is_err() {
                println!("Socket errored during send");
                println!("{:?}", result.err());

                continue;
            }

            // Check whether the response has been sent and remove the future and thereby the socket on success
            match result.unwrap() {
                Async::Ready((stream, _)) => {
                    match stream.shutdown(Shutdown::Both) {
                        Err(error) => {
                            println!("Error during socket shutdown: {:?}", error);
                        }
                        _ => {}
                    }
                    continue;
                }
                Async::NotReady => {
                    not_ready.insert(uuid, future);
                }
            }
        }

        self.unix_responses = not_ready;
    }
}
