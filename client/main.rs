use ::anyhow::Result;
use ::simplelog::{Config, LevelFilter, SimpleLogger};

use ::pueue::settings::Settings;

pub mod cli;
pub mod client;
pub mod output;

use crate::cli::{handle_cli, get_message_from_matches};
use crate::client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::new()?;
    let save_result = settings.save();

    if save_result.is_err() {
        println!("Failed saving config file.");
        println!("{:?}", save_result.err());
    }

    let matches = handle_cli();
    if matches.is_present("verbose") {
        SimpleLogger::init(LevelFilter::Info, Config::default())?;
    } else {
        SimpleLogger::init(LevelFilter::Warn, Config::default())?;
    }

    let message = get_message_from_matches(&matches)?;
    let mut client = Client::new(settings, message)?;
    client.run().await?;

    Ok(())
}
