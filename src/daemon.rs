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

    fn accept_incoming(&mut self) {
        loop {
            match self.unix_listener.poll_read() {
                Async::Ready(()) => {
                    let result = self.unix_listener.accept();
                    if result.is_err() {
                        println!("Failed to accept incoming unix connection.");
                        break;
                    }

                    let (stream, _socket_addr) = result.unwrap();
                    let buffer: Vec<u8> = vec![0; 8];

                    println!("Got new unix connection.");
                    let incoming = tokio_io::read_exact(stream, buffer).then(|result| {
                        let (stream, message) = result.unwrap();
                        let mut header = Cursor::new(message);
                        let message_size = header.read_u64::<BigEndian>().unwrap() as usize;
                        let buffer: Vec<u8> = vec![0; message_size];
                        tokio_io::read_exact(stream, buffer)
                    });
                    self.unix_incoming.push(Box::new(incoming));
                }
                Async::NotReady => break,
            }
        }
    }

    fn handle_incoming(&mut self) {
        println!("Start incoming polls");
        let len = self.unix_incoming.len();
        for i in (0..len).rev() {
            let result = self.unix_incoming[i].poll();
            println!("Polling for read {}", i);

            // Handle socket error
            if result.is_err() {
                println!("Socket errored during read");
                println!("{:?}", result.err());
                self.unix_incoming.remove(i);

                continue;
            }

            // Handle message and create response future
            match result.unwrap() {
                Async::Ready((stream, message_bytes)) => {
                    let message_result = String::from_utf8(message_bytes);
                    if message_result.is_err() {
                        println!("Didn't receive valid utf8.");
                        self.unix_incoming.remove(i);

                        continue;
                    }
                    println!("{}", message_result.unwrap());

                    let response = String::from("heyo");
                    let response_future = tokio_io::write_all(stream, response.into_bytes());
                    self.unix_response.push(Box::new(response_future));
                    self.unix_incoming.remove(i);
                }
                Async::NotReady => {}
            }
        }
        println!("End incoming polls");
    }

    fn handle_responses(&mut self) {
        println!("Start response polls");
        let len = self.unix_response.len();
        for i in (0..len).rev() {
            let result = self.unix_response[i].poll();
            println!("Polling for send {}", i);

            // Handle socket error
            if result.is_err() {
                println!("Socket errored during send");
                println!("{:?}", result.err());
                self.unix_response.remove(i);

                continue;
            }

            // Handle message and create response future
            match result.unwrap() {
                Async::Ready((_, _)) => {
                    self.unix_response.remove(i);
                }
                Async::NotReady => {}
            }
        }
        println!("End response polls");
    }
}

impl Future for Daemon {
    type Item = ();
    type Error = String;

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
