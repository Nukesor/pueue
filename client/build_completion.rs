use ::structopt::StructOpt;
use ::structopt::clap::Shell;

pub mod cli;

use crate::cli::Opt;

fn main() {
    let mut clap = Opt::clap();

    let completion_dir = "./utils/completions";
    let bin_name = "pueue";
    clap.gen_completions(bin_name, Shell::Bash, completion_dir);
    clap.gen_completions(bin_name, Shell::Fish, completion_dir);
    clap.gen_completions(bin_name, Shell::PowerShell, completion_dir);
    clap.gen_completions(bin_name, Shell::Elvish, completion_dir);
    clap.gen_completions(bin_name, Shell::Zsh, completion_dir);
}
