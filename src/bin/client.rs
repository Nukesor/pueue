extern crate pueue;
extern crate tokio_core;

use tokio_core::reactor::Core;

use pueue::client::client::Client;
use pueue::settings::Settings;

fn main() {
    let settings = Settings::new().unwrap();
    let save_result = settings.save();

    if save_result.is_err() {
        println!("Failed saving config file.");
        println!("{:?}", save_result.err());
    }

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let client = Client::new(settings, handle);

    core.run(client).unwrap();
}
