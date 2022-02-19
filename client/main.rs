use anyhow::{bail, Context, Result};
use clap::{IntoApp, Parser};
use clap_complete::{generate_to, shells};
use simplelog::{Config, LevelFilter, SimpleLogger};

use pueue_lib::settings::Settings;

pub mod cli;
pub mod client;
pub mod commands;
pub mod display;

use crate::cli::{CliArguments, Shell, SubCommand};
use crate::client::Client;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Parse commandline options.
    let opt = CliArguments::parse();

    if let Some(SubCommand::Completions {
        shell,
        output_directory,
    }) = &opt.cmd
    {
        let mut app = CliArguments::command();
        app.set_bin_name("pueue");
        let completion_result = match shell {
            Shell::Bash => generate_to(shells::Bash, &mut app, "pueue", output_directory),
            Shell::Elvish => generate_to(shells::Elvish, &mut app, "pueue", output_directory),
            Shell::Fish => generate_to(shells::Fish, &mut app, "pueue", output_directory),
            Shell::PowerShell => {
                generate_to(shells::PowerShell, &mut app, "pueue", output_directory)
            }
            Shell::Zsh => generate_to(shells::Zsh, &mut app, "pueue", output_directory),
        };
        completion_result.context(format!("Failed to generate completions for {shell:?}"))?;
        return Ok(());
    }

    // Set the verbosity level of the logger.
    let level = match opt.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    SimpleLogger::init(level, Config::default()).unwrap();

    // Try to read settings from the configuration file.
    let (mut settings, config_found) =
        Settings::read(&opt.config).context("Failed to read configuration.")?;

    // Load any requested profile.
    if let Some(profile) = &opt.profile {
        settings.load_profile(profile)?;
    }

    #[allow(deprecated)]
    if settings.daemon.groups.is_some() {
        println!(
            "Please delete the 'daemon.groups' section from your config file.\n\
            It is no longer used and groups can now only be edited via the commandline interface.\n\n\
            Attention: The first time the daemon is restarted this update, the amount of parallel tasks per group will be reset to 1!!"
        )
    }

    // Error if no configuration file can be found, as this is an indicator, that the daemon hasn't
    // been started yet.
    if !config_found {
        bail!("Couldn't find a configuration file. Did you start the daemon yet?");
    }

    // Create client to talk with the daemon and connect.
    let mut client = Client::new(settings, opt)
        .await
        .context("Failed to initialize client.")?;
    client.start().await?;

    Ok(())
}
