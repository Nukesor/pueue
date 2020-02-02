use ::structopt::clap::Shell;
use ::version_check;

include!("client/cli.rs");

fn main() {
    if !version_check::is_min_version("1.39.0").unwrap_or(false) {
        eprintln!("Pueue needs to be build with Rust version >=1.39");
        eprintln!("Please update your rust version to stable.");
        std::process::exit(1);
    }

    let mut clap = Opt::clap();

    let completion_dir = "./utils/completions";
    let bin_name = "pueue";
    clap.gen_completions(bin_name, Shell::Bash, completion_dir);
    clap.gen_completions(bin_name, Shell::Fish, completion_dir);
    clap.gen_completions(bin_name, Shell::PowerShell, completion_dir);
    clap.gen_completions(bin_name, Shell::Elvish, completion_dir);
//    clap.gen_completions(bin_name, Shell::Zsh, completion_dir);
}
