mkdir -p utils/completions
cargo run --bin pueue completions bash ./utils/completions
cargo run --bin pueue completions fish ./utils/completions
cargo run --bin pueue completions powershell ./utils/completions
cargo run --bin pueue completions elvish ./utils/completions
# Broken for now -.-
#cargo run --bin pueue completions zsh ./utils/completions
