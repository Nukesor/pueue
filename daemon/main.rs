use ::anyhow::{bail, Error, Result};
use ::simplelog::{Config, LevelFilter, SimpleLogger};
use ::std::sync::mpsc::channel;
use ::std::sync::{Arc, Mutex};
use ::std::thread;

use crate::socket::accept_incoming;
use crate::task_handler::TaskHandler;
use ::pueue::settings::Settings;
use ::pueue::state::State;

pub mod socket;
pub mod task_handler;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = SimpleLogger::init(LevelFilter::Debug, Config::default());
    let settings = Settings::new()?;
    match settings.save() {
        Err(error) => {
            let error: Error = From::from(error);
            bail!(error.context("Failed saving the config file"));
        }
        Ok(()) => {}
    };

    let state = Arc::new(Mutex::new(State::new()));

    let (sender, receiver) = channel();
    let mut task_handler = TaskHandler::new(settings.clone(), state.clone(), receiver);

    thread::spawn(move || {
        task_handler.run();
    });

    accept_incoming(settings, sender, state.clone()).await?;

    Ok(())
}
