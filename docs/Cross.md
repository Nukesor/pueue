# Cross compilation

Compilation and testing for other architectures is rather easy with `cross`.

1. Install `cargo-cross`.
1. Make sure to install `qemu`.
  - On Arch-Linux install `qemu-user-static-binfmt`.
  - On Ubuntu install `binfmt-support` and `qemu-user-static`.

Run the build/test against the target infrastructure, I.e.:

- `cross build --target=aarch64-unknown-linux-musl`
- `cross test --target=aarch64-unknown-linux-musl`
