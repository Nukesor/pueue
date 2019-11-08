use ::anyhow::{Result, bail, Error};
use ::pueue::settings::Settings;


#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::new().unwrap();
    match settings.save(){
        Err(error) => {
            let error: Error = From::from(error);
            bail!(error.context("Failed saving the config file"));
        }
        Ok(()) => {}
    };

    Ok(())
}
