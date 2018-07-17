use futures::Future;
use tokio::prelude::*;
use tokio_uds::UnixListener;
use tokio_core::reactor::Handle;

use communication::local::{get_unix_listener, UnixHandler};
use settings::Settings;

/// The daemon is center of all logic in pueue.
/// This is the single source of truth for all clients and workers.
pub struct Daemon {
    unix_listener: UnixListener,
    unix_poller: Vec<UnixHandler>,
}

impl Daemon {
    /// Create a new daemon.
    /// This function also handle the creation of other components,
    /// such as the queue, sockets and the process handler.
    pub fn new(settings: &Settings, handle: Handle) -> Self {
        let unix_listener = get_unix_listener(&settings, &handle);

        Daemon {
            unix_listener: unix_listener,
            unix_poller: Vec::new(),
        }
    }
}

impl Future for Daemon {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<()>, Self::Error> {
        // Accept all new connections
        loop {
            match self.unix_listener.poll_read() {
                Async::Ready(()) => {
                    let result = self.unix_listener.accept();
                    if result.is_err() {
                        break;
                    }
                    let (connection, _socket_addr) = result.unwrap();

                    let connection = UnixHandler{
                        connection: connection,
                        message: String::new(),
                    };

                    self.unix_poller.push(connection);
                }
                Async::NotReady => break,
            }
        }

        // Poll all connection futures.
        let len = self.unix_poller.len();
        for i in (0..len).rev() {
            let result = self.unix_poller[i].poll();
            if result.is_err() {
                println!("Socket errored");
                println!("{:?}", result.err());
                self.unix_poller.remove(i);
                continue;
            }
            match result.unwrap() {
                Async::Ready(_) => {
                    self.unix_poller.remove(i);
                    println!("ROFL");
                }
                Async::NotReady => {}
            }
        }

        // `NotReady` is returned here because the future never actually
        // completes. The server runs until it is dropped.
        Ok(Async::NotReady)
    }
}
