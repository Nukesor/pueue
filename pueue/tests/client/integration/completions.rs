use std::process::{Command, Stdio};

use assert_cmd::cargo_bin;
use rstest::rstest;

use crate::internal_prelude::*;

const NUMERIC_PARAMS: &[&str] = &[
    "task_id",
    "task_id_1",
    "task_id_2",
    "task_ids",
    "parallel_tasks",
];

/// Make sure completion for all shells work as expected.
/// This test tests writing to file.
#[rstest]
#[case("zsh")]
#[case("elvish")]
#[case("bash")]
#[case("fish")]
#[case("power-shell")]
#[case("nushell")]
#[test]
fn autocompletion_generation_to_file(#[case] shell: &'static str) -> Result<()> {
    let output = Command::new(cargo_bin!("pueue"))
        .arg("completions")
        .arg(shell)
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

/// Make sure completion for all shells work as expected.
/// This test tests writing to stdout.
#[rstest]
#[case("zsh")]
#[case("elvish")]
#[case("bash")]
#[case("fish")]
#[case("power-shell")]
#[case("nushell")]
#[test]
fn autocompletion_generation_to_stdout(#[case] shell: &'static str) -> Result<()> {
    let output = Command::new(cargo_bin!("pueue"))
        .arg("completions")
        .arg(shell)
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

/// Test that nushell completions correctly map numeric types to int instead of string.
/// This is a regression test for issue #572.
#[test]
fn nushell_numeric_types() -> Result<()> {
    let output = Command::new(cargo_bin!("pueue"))
        .arg("completions")
        .arg("nushell")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(env!("CARGO_TARGET_TMPDIR"))
        .output()
        .context("Failed to run completion generation for nushell")?;

    assert!(
        output.status.success(),
        "Completion for nushell didn't finish successfully."
    );

    let completions =
        String::from_utf8(output.stdout).context("Failed to parse nushell completions as UTF-8")?;

    // Verify that numeric parameters are typed as int, not string
    for param in NUMERIC_PARAMS {
        // Check for patterns like "task_id: int", "...task_ids: int", "task_id?: int"
        let has_int_type = completions.contains(&format!("{}: int", param))
            || completions.contains(&format!("...{}: int", param))
            || completions.contains(&format!("{}?: int", param));

        assert!(
            has_int_type,
            "Parameter '{}' should be typed as 'int' in nushell completions",
            param
        );

        // Ensure it's not typed as string
        let has_string_type = completions.contains(&format!("{}: string", param))
            || completions.contains(&format!("...{}: string", param))
            || completions.contains(&format!("{}?: string", param));

        assert!(
            !has_string_type,
            "Parameter '{}' should not be typed as 'string' in nushell completions (found in output)",
            param
        );
    }

    // Specific test for the switch command mentioned in the issue
    assert!(
        completions.contains("task_id_1: int"),
        "Switch command task_id_1 should be int type"
    );
    assert!(
        completions.contains("task_id_2: int"),
        "Switch command task_id_2 should be int type"
    );

    Ok(())
}
