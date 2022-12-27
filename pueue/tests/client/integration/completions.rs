use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use assert_cmd::prelude::*;
use rstest::rstest;

/// Make sure that the daemon's environment variables don't bleed into the spawned subprocesses.
#[rstest]
#[case("zsh")]
#[case("elvish")]
#[case("bash")]
#[case("fish")]
#[case("power-shell")]
#[test]
fn autocompletion_generation(#[case] shell: &'static str) -> Result<()> {
    let output = Command::cargo_bin("pueue")?
        .arg("completions")
        .arg("zsh")
        .arg("./")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(env!("CARGO_TARGET_TMPDIR"))
        .output()
        .context(format!("Failed to run completion generation for {shell}:"))?;

    assert!(
        output.status.success(),
        "Completion for {shell} didn't finish successfully."
    );

    Ok(())
}
