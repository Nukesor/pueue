use ::structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "Pueue daemon",
    about = "Start the daemon for pueue",
    author = "Arne Beer <contact@arne.beer>"
)]
pub struct Opt {
    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

//    /// The ip the daemon listens on. Overwrites the address in the config file
//    #[structopt(short, long)]
//    pub address: Option<String>,

    /// The port the daemon listens on. Overwrites the port in the config file
    #[structopt(short, long)]
    pub port: Option<String>,
}
