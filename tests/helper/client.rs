use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use anyhow::{bail, Context, Result};
use assert_cmd::prelude::*;

use handlebars::Handlebars;
use pueue_lib::settings::*;
use pueue_lib::task::TaskStatus;

use super::get_state;

/// Spawn a client command that connects to a specific daemon.
pub async fn run_client_command(shared: &Shared, args: &[&str]) -> Result<Output> {
    // Inject an environment variable into the pueue command.
    // This is used to ensure that the environment is properly captured and forwarded.
    let mut envs = HashMap::new();
    envs.insert("PUEUED_TEST_ENV_VARIABLE", "Test");

    run_client_command_with_env(shared, args, envs).await
}

/// Spawn a client command that connects to a specific daemon.
/// Accepts a list of environment variables that'll be injected into the client's env.
pub async fn run_client_command_with_env(
    shared: &Shared,
    args: &[&str],
    envs: HashMap<&str, &str>,
) -> Result<Output> {
    let output = Command::cargo_bin("pueue")?
        .arg("--config")
        .arg(shared.pueue_directory().join("pueue.yml").to_str().unwrap())
        .args(args)
        .envs(envs)
        .current_dir(shared.pueue_directory())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context(format!("Failed to execute pueue with {:?}", args))?;

    if !output.status.success() {
        bail!(
            "Command failed to run.\nCommand: {args:?}\n\nstdout:\n{}\n\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    }

    Ok(output)
}

/// Read the current state and extract the tasks' info into a context.
pub async fn get_task_context(settings: &Settings) -> Result<HashMap<String, String>> {
    // Get the current state
    let state = get_state(&settings.shared).await?;

    let mut context = HashMap::new();

    // Get the current daemon cwd.
    context.insert(
        "cwd".to_string(),
        settings
            .shared
            .pueue_directory()
            .to_string_lossy()
            .to_string(),
    );

    for (id, task) in state.tasks {
        let task_name = format!("task_{}", id);

        if let Some(start) = task.start {
            let formatted = start
                .format(&settings.client.status_time_format)
                .to_string();
            context.insert(format!("{task_name}_start"), formatted);
            context.insert(format!("{task_name}_start_long"), start.to_rfc2822());
        }
        if let Some(end) = task.end {
            let formatted = end.format(&settings.client.status_time_format).to_string();
            context.insert(format!("{task_name}_end"), formatted);
            context.insert(format!("{task_name}_end_long"), end.to_rfc2822());
        }

        if let TaskStatus::Stashed {
            enqueue_at: Some(enqueue_at),
        } = task.status
        {
            let enqueue_at = enqueue_at.format(&settings.client.status_time_format);
            context.insert(format!("{task_name}_enqueue_at"), enqueue_at.to_string());
        }
    }

    Ok(context)
}

/// This function takes the name of a snapshot template, applies a given context to the template
/// and compares it with a given `stdout`.
pub fn assert_stdout_matches(
    name: &str,
    stdout: Vec<u8>,
    context: HashMap<String, String>,
) -> Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("client")
        .join("snapshots")
        .join(&name);

    let stdout = String::from_utf8(stdout).context("Got invalid utf8 as stdout!")?;

    let template = read_to_string(&path);
    let template = match template {
        Ok(template) => template,
        Err(_) => {
            println!("Current stdout:\n{stdout}");
            bail!("Failed to read template file {path:?}")
        }
    };

    // Init Handlebars. We set to strict, as we want to show an error on missing variables.
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    let expected = handlebars
        .render_template(&template, &context)
        .context(format!(
            "Failed to render template for file: {name} with context {context:?}"
        ))?;

    if expected != stdout {
        println!("Expected output:\n-----\n{expected}\n-----");
        println!("\nGot output:\n-----\n{stdout}\n-----");
        println!(
            "\n{}",
            similar_asserts::SimpleDiff::from_str(&expected, &stdout, "expected", "actual")
        );
        bail!("The stdout of the command doesn't match the expected string");
    }

    Ok(())
}
