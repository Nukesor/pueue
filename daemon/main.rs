use std::process::Command;

use anyhow::Result;
use clap::Clap;
use simplelog::{Config, LevelFilter, SimpleLogger};

use pueue_daemon_lib::cli::CliArguments;
use pueue_daemon_lib::run;

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
    SimpleLogger::init(level, Config::default()).unwrap();

    run(opt.config).await
}

/// This is a simple and cheap custom fork method.
/// Simply spawn a new child with identical arguments and exit right away.
fn fork_daemon(opt: &CliArguments) -> Result<()> {
    let mut arguments = Vec::<String>::new();

    if let Some(config) = &opt.config {
        arguments.push("--config".to_string());
        arguments.push(config.to_string_lossy().into_owned());
    }

    if opt.verbose > 0 {
        arguments.push("-".to_string() + &" ".repeat(opt.verbose as usize));
    }

    Command::new("pueued").args(&arguments).spawn()?;

    println!("Pueued is now running in the background");
    Ok(())
}
