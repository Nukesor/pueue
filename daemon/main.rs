use ::anyhow::{bail, Result};
use ::simplelog::{Config, LevelFilter, SimpleLogger};
use ::std::fs::create_dir_all;
use ::std::path::Path;
use ::std::process::Command;
use ::std::sync::mpsc::channel;
use ::std::sync::{Arc, Mutex};
use ::std::thread;
use ::structopt::StructOpt;

use crate::cli::Opt;
use crate::socket::accept_incoming;
use crate::task_handler::TaskHandler;
use ::pueue::settings::Settings;
use ::pueue::state::State;

mod aliasing;
mod cli;
mod instructions;
mod platform;
mod response_helper;
mod socket;
mod streaming;
mod task_handler;

#[async_std::main]
async fn main() -> Result<()> {
    let settings = Settings::new()?;
    match settings.save() {
        Err(error) => {
            bail!(error.context("Failed saving the config file"));
        }
        Ok(()) => {}
    };

    // Parse commandline options.
    let opt = Opt::from_args();

    if opt.daemonize {
        fork_daemon(&opt)?;
    }

    // Set the verbosity level for the client app.
    if opt.verbose >= 3 {
        SimpleLogger::init(LevelFilter::Debug, Config::default())?;
    } else if opt.verbose == 2 {
        SimpleLogger::init(LevelFilter::Info, Config::default())?;
    } else if opt.verbose == 1 {
        SimpleLogger::init(LevelFilter::Warn, Config::default())?;
    } else if opt.verbose == 0 {
        SimpleLogger::init(LevelFilter::Error, Config::default())?;
    }

    init_directories(&settings.daemon.pueue_directory);

    let state = State::new(&settings);
    let state = Arc::new(Mutex::new(state));

    let (sender, receiver) = channel();
    let mut task_handler = TaskHandler::new(state.clone(), receiver);

    thread::spawn(move || {
        task_handler.run();
    });

    accept_incoming(sender, state.clone(), opt).await?;

    Ok(())
}

/// Initialize all directories needed for normal operation.
fn init_directories(path: &str) {
    // Pueue base path
    let pueue_dir = Path::new(path);
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

/// This is a simple and cheap custom fork method.
/// Simply spawn a new child with identical arguments and exit right away.
fn fork_daemon(opt: &Opt) -> Result<()> {
    let mut arguments = Vec::<String>::new();

    if let Some(port) = &opt.port {
        arguments.push("--port".to_string());
        arguments.push(port.clone());
    }

    if opt.verbose > 0 {
        arguments.push("-".to_string() + &" ".repeat(opt.verbose as usize));
    }

    Command::new("pueued").args(&arguments).spawn()?;

    println!("Pueued is now running in the background");
    std::process::exit(0);
}
