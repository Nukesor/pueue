use std::process::Command;

use anyhow::Result;
use clap::Parser;
use log::warn;
use simplelog::{Config, ConfigBuilder, LevelFilter, SimpleLogger, TermLogger, TerminalMode};

use pueue::daemon::{cli::CliArguments, run};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    // Parse commandline options.
    let opt = CliArguments::parse();

    if opt.daemonize {
        // Ordinarily this would be handled in clap, but they don't support conflicting args
        // with subcommands
        #[cfg(target_os = "windows")]
        if opt.service.is_some() {
            use clap::CommandFactory;
            let mut cmd = CliArguments::command();
            cmd.print_help()?;
            return Ok(());
        }

        return fork_daemon(&opt);
    }

    // Set the verbosity level of the logger.
    let level = match opt.verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
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

    // Init a terminal logger. If this fails for some reason, try fallback to a SimpleLogger
    if TermLogger::init(
        level,
        logger_config.clone(),
        TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    )
    .is_err()
    {
        SimpleLogger::init(level, logger_config).unwrap();
    }

    #[cfg(target_os = "windows")]
    {
        use pueue::daemon::cli::{ServiceSubcommand, ServiceSubcommandEntry};
        use pueue::daemon::service;

        if let Some(ServiceSubcommandEntry::Service(service)) = opt.service {
            match service {
                ServiceSubcommand::Run => {
                    // start service
                    service::run_service(opt.config.clone(), opt.profile.clone())?;
                    return Ok(());
                }

                ServiceSubcommand::Install => {
                    service::install_service(opt.config.clone(), opt.profile.clone())?;
                    println!("Successfully installed `pueued` Windows service");
                    return Ok(());
                }

                ServiceSubcommand::Uninstall => {
                    service::uninstall_service()?;
                    println!("Successfully uninstalled `pueued` Windows service");
                    return Ok(());
                }

                ServiceSubcommand::Start => {
                    service::start_service()?;
                    return Ok(());
                }

                ServiceSubcommand::Stop => {
                    service::stop_service()?;
                    return Ok(());
                }
            }
        }
    }

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

    let mut command = Command::new(current_exe);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
        .args(&arguments)
        .spawn()
        .expect("Failed to fork new process.");

    println!("Pueued is now running in the background");
    Ok(())
}
