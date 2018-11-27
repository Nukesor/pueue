use ::failure::Error;
use ::tokio_core::reactor::Core;

use pueue::client::client::Client;
use pueue::settings::Settings;

fn main() -> Result<(), Error> {
    let settings = Settings::new().unwrap();
    let save_result = settings.save();

    if save_result.is_err() {
        println!("Failed saving config file.");
        println!("{:?}", save_result.err());
    }

    let mut core = Core::new()?;
    let client = Client::new(settings);

    core.run(client)?;

    Ok(())
}
