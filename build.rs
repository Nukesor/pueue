use ::version_check;

fn main() {
    if !version_check::is_min_version("1.42.0").unwrap_or(false) {
        eprintln!("Pueue needs to be build with Rust version >=1.42");
        eprintln!("Please update your rust version to stable.");
        std::process::exit(1);
    }
}
