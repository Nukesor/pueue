use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueHint};

#[derive(Parser, Debug)]
#[command(name = "pueued", about = "Start the Pueue daemon", author, version)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// If this flag is set, the daemon will start and fork itself into the background.
    /// Closing the terminal won't kill the daemon any longer.
    /// This should be avoided and rather be properly done using a service manager.
    #[arg(short, long)]
    pub daemonize: bool,

    /// If provided, Pueue only uses this config file.
    /// This path can also be set via the "PUEUE_CONFIG" environment variable.
    /// The commandline option overwrites the environment variable!
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    /// The name of the profile that should be loaded from your config file.
    #[arg(short, long)]
    pub profile: Option<String>,
}
