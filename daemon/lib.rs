use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{fs::create_dir_all, path::PathBuf};

use anyhow::{bail, Result};
use crossbeam_channel::unbounded;
use log::warn;

use pueue_lib::network::certificate::create_certificates;
use pueue_lib::network::message::Message;
use pueue_lib::network::protocol::socket_cleanup;
use pueue_lib::network::secret::init_shared_secret;
use pueue_lib::settings::Settings;
use pueue_lib::state::State;

use crate::network::socket::accept_incoming;
use crate::task_handler::TaskHandler;

pub mod cli;
mod network;
mod pid;
mod platform;
mod task_handler;

/// The main entry point for the daemon logic.
/// It's basically the `main`, but publicly exported as a library.
/// That way we can properly do integration testing for the daemon.
pub async fn run(config_path: Option<PathBuf>) -> Result<()> {
    // Try to read settings from the configuration file.
    let settings = match Settings::read(&config_path) {
        Ok(settings) => settings,
        Err(_) => {
            // There's something wrong with the config file or something's missing.
            // Try to read the config and fill missing values with defaults.
            // This might be possible on version upgrade or first run.
            let settings = Settings::read_with_defaults(false, &config_path)?;

            // Since we needed to add values to the configuration, we have to save it.
            // This also creates the save file in case it didn't exist yet.
            if let Err(error) = settings.save(&config_path) {
                bail!("Failed saving config file: {:?}.", error);
            }
            settings
        }
    };

    init_directories(&settings.shared.pueue_directory());
    if !settings.shared.daemon_key().exists() && !settings.shared.daemon_cert().exists() {
        create_certificates(&settings.shared)?;
    }
    init_shared_secret(&settings.shared.shared_secret_path())?;
    pid::create_pid_file(&settings.shared.pueue_directory())?;

    let mut state = State::new(&settings, config_path.clone());
    // Restore the previous state and save any changes that might have happened during this process
    if let Err(error) = state.restore() {
        warn!("Failed to restore previous state:\n {:?}", error);
        warn!("Using clean state instead.");
    };
    state.save()?;
    let state = Arc::new(Mutex::new(state));

    let (sender, receiver) = unbounded();
    let mut task_handler = TaskHandler::new(state.clone(), receiver);

    // This section handles Shutdown via SigTerm/SigInt process signals
    // 1. Remove the unix socket (if it exists).
    // 2. Notify the TaskHandler, so it can shutdown gracefully.
    //
    // The actual program exit will be done via the TaskHandler.
    let sender_clone = sender.clone();
    let settings_clone = settings.clone();
    ctrlc::set_handler(move || {
        if let Err(error) = socket_cleanup(&settings_clone.shared) {
            println!("Failed to cleanup socket after shutdown signal.");
            println!("{}", error);
        }

        // Notify the task handler
        sender_clone
            .send(Message::DaemonShutdown)
            .expect("Failed to send Message to TaskHandler on Shutdown");
    })?;

    let orig_hook = std::panic::take_hook();
    let settings_clone = settings.clone();
    std::panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        if let Err(error) = pid::cleanup_pid_file(&settings_clone.shared.pueue_directory()) {
            println!("Failed to cleanup pid after panic.");
            println!("{}", error);
        }
        if let Err(error) = socket_cleanup(&settings_clone.shared) {
            println!("Failed to cleanup socket after panic.");
            println!("{}", error);
        }

        std::process::exit(1);
    }));

    std::thread::spawn(move || {
        task_handler.run();
    });

    accept_incoming(sender, state.clone()).await?;

    Ok(())
}

/// Initialize all directories needed for normal operation.
fn init_directories(pueue_dir: &Path) {
    // Pueue base path
    if !pueue_dir.exists() {
        if let Err(error) = create_dir_all(&pueue_dir) {
            panic!(
                "Failed to create main directory at {:?} error: {:?}",
                pueue_dir, error
            );
        }
    }

    // Task log dir
    let log_dir = pueue_dir.join("log");
    if !log_dir.exists() {
        if let Err(error) = create_dir_all(&log_dir) {
            panic!(
                "Failed to create log directory at {:?} error: {:?}",
                log_dir, error
            );
        }
    }

    // Task certs dir
    let certs_dir = pueue_dir.join("certs");
    if !certs_dir.exists() {
        if let Err(error) = create_dir_all(&certs_dir) {
            panic!(
                "Failed to create certificate directory at {:?} error: {:?}",
                certs_dir, error
            );
        }
    }

    // Task log dir
    let logs_dir = pueue_dir.join("task_logs");
    if !logs_dir.exists() {
        if let Err(error) = create_dir_all(&logs_dir) {
            panic!(
                "Failed to create task logs directory at {:?} error: {:?}",
                logs_dir, error
            );
        }
    }
}
