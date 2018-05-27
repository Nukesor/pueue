use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread::sleep;
use std::time::{Duration, Instant};

use communication::local::get_unix_listener;
use settings::Settings;

/// The daemon is center of all logic in pueue.
/// This is the single source of truth for all clients and workers.
pub struct Daemon {
    unix_listener: UnixListener,
    last_tick: Instant,
}

impl Daemon {
    /// Create a new daemon.
    /// This function also handle the creation of other components,
    /// such as the queue, sockets and the process handler.
    pub fn new(settings: &Settings) -> Self {
        let unix_listener = get_unix_listener(&settings);

        Daemon {
            unix_listener: unix_listener,
            last_tick: Instant::now(),
        }
    }

    /// Check if there are any incoming connections on any of the open sockets.
    /// Incoming requests are handled in `self.handle_request`.
    pub fn check_connections(&self) {
        // accept connections and process them, spawning a new thread for each one
        for stream in self.unix_listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut request = String::new();
                    let result = stream.read_to_string(&mut request);
                    if result.is_err() {
                        println!("Failed to receive message from local client. Ignoring.");
                    }

                    self.handle_request(&request);
                }
                Err(err) => {
                    println!("Local client connection error. {:?}.", err);
                    continue;
                }
            }
        }
    }

    pub fn handle_request(&self, request: &str) {
        println!("{}", request);
    }

    /// Start the daemon.
    /// Continuously check for finished process and new messages from clients
    pub fn start(&mut self) {
        loop {
            self.check_connections();
            self.sleep();
        }
    }

    /// Consistently tick. If a tick takes longer than the
    /// given threshold an error warning is printed
    pub fn sleep(&mut self) {
        let threshold = Duration::from_millis(500);

        let result = threshold.checked_sub(self.last_tick.elapsed());
        match result {
            Some(duration) => {
                println!("Sleeping for {} milliseconds.", duration.subsec_millis());
                sleep(duration);
            }
            None => println!(
                "Tick took {} milliseconds.",
                self.last_tick.elapsed().subsec_millis()
            ),
        }

        println!("tick");
        self.last_tick = Instant::now();
    }
}
