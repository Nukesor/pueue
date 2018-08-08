use byteorder::{BigEndian, WriteBytesExt};
use futures::Future;
use std::str;
use tokio::prelude::*;
use tokio_core::reactor::Core;
use tokio_io::io as tokio_io;

use communication::local::get_unix_stream;
use settings::Settings;
use client::cli::get_app;

/// The client
pub struct Client {}

impl Client {
    pub fn call(settings: &Settings) {
        // Create a new tokio core
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let unix_stream = get_unix_stream(&settings, &handle);

        // Get commandline arguments
        let matches = get_app();

        // Get command
        let message = matches.value_of("command").unwrap();
        let command_type = 1 as u64;

        // Prepare command for transfer and determine message byte length
        let payload= message.as_bytes();
        let byte_size = payload.len() as u64;

        let mut header = vec![];
        header.write_u64::<BigEndian>(byte_size).unwrap();
        //header.write_u64::<BigEndian>(command_type).unwrap();

        // Send the request size header and the request to the header
        let process = tokio_io::write_all(unix_stream, header)
            .and_then(|(stream, _written)| tokio_io::write_all(stream, payload))
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
