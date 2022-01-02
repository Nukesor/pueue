use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(
    name = "Pueue daemon",
    about = "Start the Pueue daemon",
    author,
    version
)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// If this flag is set, the daemon will start and fork itself into the background.
    /// Closing the terminal won't kill the daemon any longer.
    /// This should be avoided and rather be properly done using a service manager.
    #[clap(short, long)]
    pub daemonize: bool,

    /// Path to a specific pueue config file to use.
    /// This ignores all other config files.
    #[clap(short, long)]
    pub config: Option<PathBuf>,
}
