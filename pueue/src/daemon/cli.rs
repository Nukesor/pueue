use std::path::PathBuf;

#[cfg(target_os = "windows")]
use clap::Subcommand;
use clap::{ArgAction, Parser, ValueHint};

#[derive(Parser, Debug)]
#[command(name = "pueued", about = "Start the Pueue daemon", author, version)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// If this flag is set, the daemon will start and fork itself into the background.
    ///
    /// Beware: Closing the terminal won't kill the daemon any longer.
    /// This should be avoided and rather be properly done using a service manager.
    #[arg(short, long)]
    pub daemonize: bool,

    /// If provided, Pueue only uses this config file.
    ///
    /// This path can also be set via the $PUEUE_CONFIG_PATH environment variable.
    /// The commandline option overwrites the environment variable!
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    /// The name of the profile that should be loaded from your config file.
    #[arg(short, long)]
    pub profile: Option<String>,

    #[cfg(target_os = "windows")]
    #[command(subcommand)]
    pub service: Option<ServiceSubcommandEntry>,
}

#[cfg(target_os = "windows")]
#[derive(Copy, Clone, Debug, Subcommand)]
pub enum ServiceSubcommandEntry {
    /// Manage the Windows Service.
    #[command(subcommand)]
    Service(ServiceSubcommand),
}

#[cfg(target_os = "windows")]
#[derive(Copy, Clone, Debug, Subcommand)]
pub enum ServiceSubcommand {
    /// Run the Windows service. This command is internal and should never
    /// be used.
    Run,
    /// Install as a Windows service.
    ///
    /// Once installed, you must not move the binary, otherwise the Windows
    /// service will not be able to find it. If you wish to move the binary,
    /// first uninstall the service, move the binary, then install the service
    /// again.
    Install,
    /// Uninstall the service.
    Uninstall,
    /// Start the service.
    Start,
    /// Stop the service.
    Stop,
}
