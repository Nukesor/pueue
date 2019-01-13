use tokio;

use pueue::daemon::daemon::Daemon;
use pueue::settings::Settings;

fn main() {
    let settings = Settings::new().unwrap();
    let save_result = settings.save();

    if save_result.is_err() {
        println!("Failed saving config file.");
        println!("{:?}", save_result.err());
    }

    let daemon = Daemon::new(&settings);

    tokio::run(daemon);
}
