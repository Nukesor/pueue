use std::str;
use futures::Future;
use tokio::prelude::*;
use tokio_core::reactor::Core;
use tokio_io::io as tokio_io;

use settings::Settings;
use communication::local::get_unix_stream;

/// The client
pub struct Client {}

impl Client {
    pub fn answer(settings: &Settings) {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let unix_stream = get_unix_stream(&settings, &handle);

        let process = tokio_io::write_all(unix_stream, b"hi?")
            .and_then(|(stream, _written)| {
                println!("Sent it. Now I'm reading.");
                tokio_io::read_to_end(stream, Vec::new())
            })
        .map(|(_stream, response_bytes)| {
            let response_result = str::from_utf8(&response_bytes);
            if response_result.is_err() {
                println!("Didn't receive valid utf8.")
            } else {
                println!("{}", response_result.unwrap());
                println!("wtf");
            }

            return Async::Ready(());
        });

        let _ = core.run(process);
    }
}
