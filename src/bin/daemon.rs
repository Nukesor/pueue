extern crate daemonize;
extern crate pueue;
extern crate tokio;
extern crate tokio_core;
extern crate users;

use tokio::prelude::*;
use tokio_core::reactor::Core;

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

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let daemon = Daemon::new(&settings, handle);

    core.run(daemon).unwrap();
}
