use ::anyhow::Result;
use ::simplelog::{Config, LevelFilter, SimpleLogger};

use crate::client::Client;
use ::pueue::settings::Settings;

pub mod cli;
pub mod client;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
    let settings = Settings::new()?;
    let save_result = settings.save();

    if save_result.is_err() {
        println!("Failed saving config file.");
        println!("{:?}", save_result.err());
    }

    let mut client = Client::new(settings)?;

    client.run().await?;

    Ok(())
}
