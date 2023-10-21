# Bump all deps, including incompatible version upgrades
bump:
    just ensure_installed upgrade
    cargo update
    cargo upgrade --incompatible
    cargo test --workspace

# Run the test suite with nexttest
nextest:
    just ensure_installed nextest
    cargo nextest run --workspace

ensure_installed *args:
    #!/bin/bash
    cargo --list | grep -q {{ args }}
    if [[ $? -ne 0 ]]; then
        echo "error: cargo-{{ args }} is not installed"
        exit 1
    fi

lint:
    cargo fmt
    cargo clippy --all --tests
