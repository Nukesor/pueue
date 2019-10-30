use ::anyhow::Result;

use ::pueue::client::client::Client;
use ::pueue::settings::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::new().unwrap();
    let save_result = settings.save();

    if save_result.is_err() {
        println!("Failed saving config file.");
        println!("{:?}", save_result.err());
    }

    let mut client = Client::new(settings)?;

    client.run().await?;

    Ok(())
}
