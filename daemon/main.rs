use ::anyhow::{bail, Error, Result};
use ::simplelog::{Config, LevelFilter, SimpleLogger};
use ::std::sync::mpsc::channel;
use ::std::sync::{Arc, Mutex};
use ::std::thread;
use ::std::path::Path;
use ::std::fs::create_dir_all;

use crate::socket::accept_incoming;
use crate::task_handler::TaskHandler;
use ::pueue::settings::Settings;
use ::pueue::state::State;

pub mod socket;
pub mod task_handler;

#[async_std::main]
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

    init_directories(&settings.daemon.pueue_directory);

    let mut state = State::new(&settings);
    state.restore();
    let state = Arc::new(Mutex::new(state));

    let (sender, receiver) = channel();
    let mut task_handler = TaskHandler::new(settings.clone(), state.clone(), receiver);

    thread::spawn(move || {
        task_handler.run();
    });

    accept_incoming(settings, sender, state.clone()).await?;

    Ok(())
}


/// Initialize all directories needed for normal operation
pub fn init_directories(path: &String) {
    let pueue_dir = Path::new(path);
    if !pueue_dir.exists() {
        if let Err(error) = create_dir_all(&pueue_dir) {
            panic!("Failed to create main directory at {:?} error: {:?}", pueue_dir, error);
        }
    }
    let log_dir = pueue_dir.join("log");
    if !log_dir.exists() {
        if let Err(error) = create_dir_all(&log_dir) {
            panic!("Failed to create log directory at {:?} error: {:?}", log_dir, error);
        }
    }

    let temp_dir = pueue_dir.join("temp");
    if !temp_dir.exists() {
        if let Err(error) = create_dir_all(&temp_dir) {
            panic!("Failed to create temp directory at {:?} error: {:?}", temp_dir, error);
        }
    }
}
