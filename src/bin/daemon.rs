use ::std::sync::mpsc::channel;
use ::std::sync::{Mutex, Arc};
use ::std::thread;
use ::anyhow::{Result, bail, Error};

use ::pueue::settings::Settings;
use ::pueue::daemon::state::State;
use ::pueue::daemon::task_handler::TaskHandler;
use ::pueue::daemon::socket_handler::accept_incoming;


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

    let state = Arc::new(Mutex::new(State::new()));

    let (sender, receiver) = channel();
    let mut task_handler = TaskHandler::new(state, receiver);

    thread::spawn(move || {
        task_handler.run();
    });

    accept_incoming(settings, sender).await;

    Ok(())
}
