extern crate daemonize;
extern crate pueue;
extern crate users;
extern crate tokio;

use tokio::prelude::*;

//use daemonize::{Daemonize};
//use users::{get_user_by_uid, get_current_uid};

use pueue::daemon::Daemon;
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
