use std::process::Command;

use anyhow::Result;
use clap::Parser;
use log::warn;
use simplelog::{Config, ConfigBuilder, LevelFilter, SimpleLogger};

use pueue::daemon::cli::CliArguments;
use pueue::daemon::run;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    // Parse commandline options.
    let opt = CliArguments::parse();

    if opt.daemonize {
        return fork_daemon(&opt);
    }

    // Set the verbosity level of the logger.
    let level = match opt.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };

    // Try to initialize the logger with the timezone set to the Local time of the machine.
    let mut builder = ConfigBuilder::new();
    let logger_config = match builder.set_time_offset_to_local() {
        Err(_) => {
            warn!("Failed to determine the local time of this machine. Fallback to UTC.");
            Config::default()
        }
        Ok(builder) => builder.build(),
    };

    SimpleLogger::init(level, logger_config).unwrap();

    run(opt.config, opt.profile, false).await
}

/// This is a simple and cheap custom fork method.
/// Simply spawn a new child with identical arguments and exit right away.
fn fork_daemon(opt: &CliArguments) -> Result<()> {
    let mut arguments = Vec::<String>::new();

    if let Some(config) = &opt.config {
        arguments.push("--config".to_string());
        arguments.push(config.to_string_lossy().into_owned());
    }

    if let Some(profile) = &opt.profile {
        arguments.push("--profile".to_string());
        arguments.push(profile.clone());
    }

    if opt.verbose > 0 {
        arguments.push("-".to_string() + &"v".repeat(opt.verbose as usize));
    }

    // Try to get the path to the current binary, since it may not be in the $PATH.
    // If we cannot detect it (for some unknown reason), fallback to the raw `pueued` binary name.
    let current_exe = if let Ok(path) = std::env::current_exe() {
        path.to_string_lossy().clone().to_string()
    } else {
        println!("Couldn't detect path of current binary. Falling back to 'pueue' in $PATH");
        "pueued".to_string()
    };

    Command::new(current_exe)
        .args(&arguments)
        .spawn()
        .expect("Failed to fork new process.");

    println!("Pueued is now running in the background");
    Ok(())
}
