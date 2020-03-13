use ::std::fs::create_dir_all;
use ::std::path::Path;
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

    let completion_path = "./utils/completions";
    let completion_dir = Path::new(completion_path).to_path_buf();
    // Create the config dir, if it doesn't exist yet
    if !completion_dir.exists() {
        create_dir_all(&completion_dir).expect("Couldn't create .utils/completion directory");
    }

    let bin_name = "pueue";
    clap.gen_completions(bin_name, Shell::Bash, completion_path);
    clap.gen_completions(bin_name, Shell::Fish, completion_path);
    clap.gen_completions(bin_name, Shell::PowerShell, completion_path);
    clap.gen_completions(bin_name, Shell::Elvish, completion_path);
    //    clap.gen_completions(bin_name, Shell::Zsh, completion_path);
}
