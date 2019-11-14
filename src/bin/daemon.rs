use ::anyhow::{bail, Error, Result};
use ::std::sync::mpsc::channel;
use ::std::sync::{Arc, Mutex};
use ::std::thread;
use ::simplelog::{Config, LevelFilter, SimpleLogger};

use ::pueue::daemon::socket::accept_incoming;
use ::pueue::daemon::state::State;
use ::pueue::daemon::task::handler::TaskHandler;
use ::pueue::settings::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
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
