use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "Pueue daemon",
    about = "Start the daemon for pueue",
    author = "Arne Beer <contact@arne.beer>"
)]
pub struct Opt {
    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// If this flag is set, the daemon will start and fork itself into the background.
    /// Closing the terminal won't kill the daemon any longer.
    /// This should be avoided and rather be properly done using a service manager.
    #[structopt(short, long)]
    pub daemonize: bool,

    /// The port the daemon listens on. Overwrites the port in the config file.
    #[structopt(short, long)]
    pub port: Option<String>,
}
