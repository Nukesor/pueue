use ::byteorder::{BigEndian, ReadBytesExt};
use ::failure::Error;
use ::futures::prelude::*;
use ::futures::Future;
use ::std::collections::HashMap;
use ::std::io::Cursor;
use ::tokio::io as tokio_io;
use ::tokio_uds::{UnixListener, UnixStream};
use ::uuid::Uuid;

use crate::communication::local::*;
use crate::communication::message::*;
use crate::settings::Settings;


pub struct SocketHandler {
    unix_listener: UnixListener,
    unix_incoming:
        Vec<Box<dyn Future<Item = (MessageType, String, UnixStream), Error = Error> + Send>>,
    unix_responses: HashMap<Uuid, Box<dyn Future<Item = (UnixStream, Vec<u8>), Error = Error> + Send>>,
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
                    let incoming = tokio_io::read_exact(stream, vec![0; 16])
                        .then(|result| {
                            let (stream, header) = result?;

                            // Extract the instruction size from the header bytes
                            let mut header = Cursor::new(header);
                            let instruction_size = header.read_u64::<BigEndian>().unwrap() as usize;
                            let instruction_index =
                                header.read_u64::<BigEndian>().unwrap() as usize;

                            // Try to resolve the instruction index
                            // If we got an invalid instruction index, c
                            let instruction_type = get_message_type(instruction_index)?;

                            Ok(ReceiveInstruction {
                                instruction_type: instruction_type,
                                read_instruction_future: Box::new(
                                    tokio_io::read_exact(stream, vec![0; instruction_size])
                                        .map_err(|error| Error::from(error)),
                                ),
                            })
                        })
                        .and_then(|future| future);
                    self.unix_incoming.push(Box::new(incoming));
                }
                Async::NotReady => break,
            }
        }
    }

    /// Continuously poll the existing incoming futures.
    /// In case we received an instruction, handle it and create a response future.
    /// The response future is added to unix_responses and handled in a separate function.
    pub fn handle_incoming(&mut self) -> HashMap<MessageType, String> {
        let mut instructions: HashMap<MessageType, String> = HashMap::new();
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
                Async::Ready((instruction_type, instruction, stream)) => {
                    println!("{:?}", instruction_type);
                    println!("{}", instruction);
                    let hash_uuid = Uuid::new_v4();
                    self.unix_sockets.insert(hash_uuid, stream);
                    self.unix_incoming.remove(i);

                    instructions.insert(instruction_type, instruction);

                }
                Async::NotReady => {}
            }
        }

        instructions
    }

    /// Send a message to a specific unix socket
    /// The uuid of the socket is contained inside the Message
    pub fn send_or_queue_message(&mut self, message: Message) {
        if self.can_be_responded_to(&message.socket_uuid) {
            self.send_message(message)
        } else if self.is_sending(&message.socket_uuid) {
            self.unix_response_queue.push(message)
        } else {
            println!("Cannot send message. The unix socket doesn't seem to exist any longer.");
            if let Ok(message) = serde_json::to_string_pretty(&message) {
                println!("{}", message);
            }
        }
    }

    /// Create the response future for this message.
    fn send_message(&mut self, message: Message) {
        let stream = self.unix_sockets.remove(&message.socket_uuid).expect("Tried to remove non-existing unix socket.");
        if let Ok(response) = serde_json::to_string(&message) {
            let response_future = tokio_io::write_all(stream, response.into_bytes());
            self.unix_responses.insert(message.socket_uuid, Box::new(
                response_future.map_err(|error| Error::from(error)),
            ));
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

    /// Send the response to the client.
    pub fn handle_responses(&mut self) {
        let mut to_remove = Vec::new();
        let mut to_reuse = Vec::new();
        for (uuid, future) in self.unix_responses.iter_mut() {
            let result = future.poll();

            // Handle socket error
            if result.is_err() {
                println!("Socket errored during send");
                println!("{:?}", result.err());
                to_remove.push(uuid.clone());

                continue;
            }

            // Check whether the response has been sent and remove the future and thereby the socket on success
            match result.unwrap() {
                Async::Ready((_, _)) => {
                    to_reuse.push(uuid);
                }
                Async::NotReady => {}
            }
        }

        // Remove all sockets that errored in some kind of way.
        for uuid in &to_remove {
            self.unix_responses.remove(uuid);
        }

        // Add all sockets to the unix_sockets HashMap for further usage.
        for uuid in &to_remove {
            if let Some(mut future) = self.unix_responses.remove(uuid) {
                if let Ok(Async::Ready((stream, _))) = future.poll() {
                    self.unix_sockets.insert(*uuid, stream);
                } else {
                    // TODO: Error handling
                    println!("A future should be ready but isn't");
                }
            } else {
                // TODO: Error handling
                println!("Failed to get socket from unix_responses");
            }
        }
    }
}
