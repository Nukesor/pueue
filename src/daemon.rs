use byteorder::{BigEndian, ReadBytesExt};
use futures::Future;
use std::io::Cursor;
use std::io::Error as io_Error;
use tokio::io as tokio_io;
use tokio::prelude::*;
use tokio_core::reactor::Handle;
use tokio_uds::{UnixListener, UnixStream};

use communication::local::get_unix_listener;
use settings::Settings;

/// The daemon is center of all logic in pueue.
/// This is the single source of truth for all clients and workers.
pub struct Daemon {
    unix_listener: UnixListener,
    unix_incoming: Vec<Box<Future<Item = (UnixStream, Vec<u8>), Error = io_Error> + Send>>,
    unix_response: Vec<Box<Future<Item = (UnixStream, Vec<u8>), Error = io_Error> + Send>>,
}

impl Daemon {
    /// Create a new daemon.
    /// This function also handle the creation of other components,
    /// such as the queue, sockets and the process handler.
    pub fn new(settings: &Settings, handle: Handle) -> Self {
        let unix_listener = get_unix_listener(&settings, &handle);

        Daemon {
            unix_listener: unix_listener,
            unix_incoming: Vec::new(),
            unix_response: Vec::new(),
        }
    }

    /// Poll the unix listener and accept new incoming connections
    /// Create a new future for receiving the message and add it to unix_incoming
    fn accept_incoming(&mut self) {
        loop {
            // Poll if we have a new incoming connection.
            // In case we don't, break the loop
            match self.unix_listener.poll_read() {
                Async::Ready(()) => {
                    // Accept new connection
                    let result = self.unix_listener.accept();

                    // Check if we can connect otherwise continue the loop
                    if result.is_err() {
                        println!("Failed to accept incoming unix connection.");
                        println!("{:?}", result.err());
                        continue;
                    }
                    let (stream, _socket_addr) = result.unwrap();

                    // First read the 8byte header to determine the size of the message
                    let incoming = tokio_io::read_exact(stream, vec![0; 8]).then(|result| {
                        let (stream, message) = result.unwrap();

                        // Extract the message size from the header bytes
                        let mut header = Cursor::new(message);
                        let message_size = header.read_u64::<BigEndian>().unwrap() as usize;

                        // Read the message
                        tokio_io::read_exact(stream, vec![0; message_size])
                    });
                    self.unix_incoming.push(Box::new(incoming));
                }
                Async::NotReady => break,
            }
        }
    }

    /// Continuously poll the existing incoming futures.
    /// In case we received a message, handle it and create a response future.
    /// The response future is added to unix_response and handled in a separate function.
    fn handle_incoming(&mut self) {
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

            // We received a message from a client. Handle it
            match result.unwrap() {
                Async::Ready((stream, message_bytes)) => {
                    // Extract message and handle invalid utf8
                    let message_result = String::from_utf8(message_bytes);
                    if message_result.is_err() {
                        println!("Didn't receive valid utf8.");
                        self.unix_incoming.remove(i);

                        continue;
                    }

                    println!("{}", message_result.unwrap());
                    // Create a future for sending the response.
                    let response = String::from("rofl");
                    let response_future = tokio_io::write_all(stream, response.into_bytes());
                    self.unix_response.push(Box::new(response_future));
                    self.unix_incoming.remove(i);
                }
                Async::NotReady => {}
            }
        }
    }

    /// Send the response to the client.
    fn handle_responses(&mut self) {
        let len = self.unix_response.len();
        for i in (0..len).rev() {
            let result = self.unix_response[i].poll();

            // Handle socket error
            if result.is_err() {
                println!("Socket errored during send");
                println!("{:?}", result.err());
                self.unix_response.remove(i);

                continue;
            }

            // Check whether the response has been sent and remove the future and thereby the socket on success
            match result.unwrap() {
                Async::Ready((_, _)) => {
                    self.unix_response.remove(i);
                }
                Async::NotReady => {}
            }
        }
    }
}

impl Future for Daemon {
    type Item = ();
    type Error = String;

    /// The poll function of the daemon.
    /// This is continuously called by the Tokio core.
    fn poll(&mut self) -> Result<Async<()>, Self::Error> {
        // Accept all new connections
        self.accept_incoming();

        // Poll all connection futures and handle the received message.
        self.handle_incoming();

        self.handle_responses();

        // `NotReady` is returned here because the future never actually
        // completes. The server runs until it is dropped.
        Ok(Async::NotReady)
    }
}
