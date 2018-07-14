extern crate pueue;
extern crate tokio_core;

use std::io::prelude::*;
use tokio_core::reactor::Core;

use pueue::communication::local::get_unix_stream;
use pueue::settings::Settings;

fn main() {
    let settings = Settings::new().unwrap();
    let save_result = settings.save();

    if save_result.is_err() {
        println!("Failed saving config file.");
        println!("{:?}", save_result.err());
    }


    let core = Core::new().unwrap();
    let handle = core.handle();
    let mut unix_stream = get_unix_stream(&settings, &handle);

    unix_stream.write_all(b"hello world").unwrap();
    let mut response = String::new();
    unix_stream.read_to_string(&mut response).unwrap();
    println!("{}", response);
}
