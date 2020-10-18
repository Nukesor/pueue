use anyhow::Result;
use simplelog::{Config, LevelFilter, SimpleLogger};
use structopt::StructOpt;

use pueue::settings::Settings;

pub mod cli;
pub mod client;
pub mod commands;
pub mod output;
pub mod output_helper;

use crate::cli::{Opt, SubCommand};
use crate::client::Client;

#[async_std::main]
async fn main() -> Result<()> {
    // Parse commandline options.
    let opt = Opt::from_args();

    if let SubCommand::Completions {
        shell,
        output_directory,
    } = &opt.cmd
    {
        let mut clap = Opt::clap();
        clap.gen_completions("pueue", *shell, output_directory);
        return Ok(());
    }

    // Set the verbosity level of the logger.
    if opt.verbose >= 3 {
        SimpleLogger::init(LevelFilter::Debug, Config::default())?;
    } else if opt.verbose == 2 {
        SimpleLogger::init(LevelFilter::Info, Config::default())?;
    } else if opt.verbose == 1 {
        SimpleLogger::init(LevelFilter::Warn, Config::default())?;
    } else if opt.verbose == 0 {
        SimpleLogger::init(LevelFilter::Error, Config::default())?;
    }

    // Try to read settings from the configuration file.
    let settings = Settings::new(true)?;

    // Create client to talk with the daemon and connect.
    let mut client = Client::new(settings, opt).await?;
    client.start().await?;

    Ok(())
}
