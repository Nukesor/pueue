use ::failure::Error;
use ::tokio::prelude::*;
use ::tokio_core::reactor::Core;

//use daemonize::{Daemonize};
//use users::{get_user_by_uid, get_current_uid};

use pueue::daemon::daemon::Daemon;
use pueue::settings::Settings;

fn main() -> Result<(), Error> {
    let settings = Settings::new().unwrap();
    let save_result = settings.save();

    if save_result.is_err() {
        println!("Failed saving config file.");
        println!("{:?}", save_result.err());
    }

    let mut core = Core::new()?;
    let daemon = Daemon::new(&settings);

    core.run(daemon)?;

    Ok(())
}
