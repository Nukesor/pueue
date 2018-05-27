extern crate pueue;

use pueue::communication::local::get_unix_stream;
use pueue::settings::Settings;

fn main() {
    let settings = Settings::new().unwrap();
    let save_result = settings.save();

    if save_result.is_err() {
        println!("Failed saving config file.");
        println!("{:?}", save_result.err());
    }

    let mut unix_listener = get_unix_stream(&settings);
}
