mkdir -p utils/completions
cargo run --release --locked --bin pueue completions bash ./utils/completions
cargo run --release --locked --bin pueue completions fish ./utils/completions
cargo run --release --locked --bin pueue completions powershell ./utils/completions
cargo run --release --locked --bin pueue completions elvish ./utils/completions
cargo run --release --locked --bin pueue completions zsh ./utils/completions
