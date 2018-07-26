use byteorder::{BigEndian, WriteBytesExt};
use futures::Future;
use std::str;
use tokio::prelude::*;
use tokio_core::reactor::Core;
use tokio_io::io as tokio_io;

use communication::local::get_unix_stream;
use settings::Settings;

/// The client
pub struct Client {}

impl Client {
    pub fn call(settings: &Settings) {
        // Create a new tokio core
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let unix_stream = get_unix_stream(&settings, &handle);

        let message = b"hi omfg im so happy.?";
        let byte_size = message.len() as u64;

        let mut header = vec![];
        header.write_u64::<BigEndian>(byte_size).unwrap();

        // Send the request size header and the request to the header
        let process = tokio_io::write_all(unix_stream, header)
            .and_then(|(stream, _written)| tokio_io::write_all(stream, message))
            // Now receive the response until the connection closes.
            .and_then(|(stream, _written)| {
                tokio_io::read_to_end(stream, Vec::new())
            })
        // Process the response
        .map(|(_stream, response_bytes)| {
            let response_result = str::from_utf8(&response_bytes);
            if response_result.is_err() {
                println!("Didn't receive valid utf8.")
            } else {
                println!("{}", response_result.unwrap());
            }

            return Async::Ready(());
        });

        let _ = core.run(process);
    }
}
