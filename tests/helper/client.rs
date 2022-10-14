use std::borrow::Cow;
use std::collections::HashMap;
use std::env::temp_dir;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use anyhow::{bail, Context, Result};
use assert_cmd::prelude::*;

use chrono::Local;
use handlebars::Handlebars;
use pueue_lib::settings::*;
use pueue_lib::task::TaskStatus;

use super::get_state;

/// Spawn a client command that connects to a specific daemon.
pub fn run_client_command(shared: &Shared, args: &[&str]) -> Result<Output> {
    // Inject an environment variable into the pueue command.
    // This is used to ensure that the environment is properly captured and forwarded.
    let mut envs = HashMap::new();
    envs.insert("PUEUED_TEST_ENV_VARIABLE", "Test");

    run_client_command_with_env(shared, args, envs)
}

/// Spawn a client command that connects to a specific daemon.
/// Accepts a list of environment variables that'll be injected into the client's env.
pub fn run_client_command_with_env(
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
            // Use datetime format for datetimes that aren't today.
            let format = if start.date() == Local::today() {
                &settings.client.status_time_format
            } else {
                &settings.client.status_datetime_format
            };

            let formatted = start.format(format).to_string();
            context.insert(format!("{task_name}_start"), formatted);
            context.insert(format!("{task_name}_start_long"), start.to_rfc2822());
        }
        if let Some(end) = task.end {
            // Use datetime format for datetimes that aren't today.
            let format = if end.date() == Local::today() {
                &settings.client.status_time_format
            } else {
                &settings.client.status_datetime_format
            };

            let formatted = end.format(format).to_string();
            context.insert(format!("{task_name}_end"), formatted);
            context.insert(format!("{task_name}_end_long"), end.to_rfc2822());
        }
        if let Some(label) = &task.label {
            context.insert(format!("{task_name}_label"), label.to_string());
        }

        if let TaskStatus::Stashed {
            enqueue_at: Some(enqueue_at),
        } = task.status
        {
            // Use datetime format for datetimes that aren't today.
            let format = if enqueue_at.date() == Local::today() {
                &settings.client.status_time_format
            } else {
                &settings.client.status_datetime_format
            };

            let enqueue_at = enqueue_at.format(format);
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
        .join(name);

    let actual = String::from_utf8(stdout).context("Got invalid utf8 as stdout!")?;

    let template = read_to_string(&path);
    let template = match template {
        Ok(template) => template,
        Err(_) => {
            println!("Actual output:\n{actual}");
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

    let expected = canonicalize_snapshot(expected, None);
    let path_column_width = find_path_column(&expected);
    let actual = canonicalize_snapshot(actual, path_column_width);

    if expected != actual {
        println!("Expected output:\n-----\n{expected}\n-----");
        println!("\nGot output:\n-----\n{actual}\n-----");
        println!(
            "\n{}",
            similar_asserts::SimpleDiff::from_str(&expected, &actual, "expected", "actual")
        );
        bail!("The stdout of the command doesn't match the expected string");
    }

    Ok(())
}

fn is_table(line: &str) -> bool {
    line.chars().all(|c| c == '\u{2500}')
}

/// Find the position and length of the Path column in the expected output
/// Path must be a space-separate word on first line following a table starting line.
fn find_path_column(output: &str) -> Option<(usize, usize)> {
    let header = output.lines().skip_while(|&line| !is_table(line)).nth(1)?;
    // Scan through the columns until we find the Path column, then produce its offset
    // and length (including whitespace padding).
    // colwidth doubles as a flag; if it is 0 we have not found the Path label (yet).
    let mut colwidth = 0;
    let mut offset = 0;
    for chunk in header.split_inclusive(char::is_whitespace) {
        match (colwidth, chunk) {
            (0, "Path") | (0, "Path ") | (1.., " ") => {
                // We found the Path column, accumulate column width
                colwidth += chunk.len();
            }
            (0, _) => {
                // We haven't yet found the Path label, accumulate column offset.
                offset += chunk.len();
            }
            _ => {
                // Path column has ended, we have reached the next label
                break;
            }
        }
    }
    if colwidth == 0 {
        None
    } else {
        Some((offset, colwidth))
    }
}

/// Canonicalize test and template outputs to handle expected differences.
/// If path_column is given, use the width to trim the Path column in this output.
fn canonicalize_snapshot(output: String, path_column: Option<(usize, usize)>) -> String {
    // Replace the temporary path with a symbolic reference, both the base
    // temporary directory and its canonical path (which on some platforms can differ)
    // These replacements should only apply to the Path column.
    const TMPVAR: &str = "$TMP";
    let tmp = temp_dir();
    let tmp_canonical = std::fs::canonicalize(&tmp).unwrap();
    let replacements = vec![
        (tmp_canonical.to_string_lossy(), &TMPVAR),
        (tmp.to_string_lossy(), &TMPVAR),
    ];

    // Set optional path column information to configure a line scan operationn below.
    let trim_path_col = match path_column {
        // Expected output has no Path column, nothing to trim here.
        None => None,
        // Determine the output Path column to see if trimming is needed.
        Some((_, target_width)) => match find_path_column(&output) {
            Some((col, actual_width)) if actual_width > target_width => {
                Some((col + target_width, col + actual_width))
            }
            // Output has no Path column or the column is not wider than the expected width
            _ => None,
        },
    };

    output
        .lines()
        .map(|line| {
            // - Trim all trailing whitespace and apply Path column replacements
            let mut tmp = Cow::from(line.trim_end());
            let before = tmp.len();
            for (from, to) in replacements.iter() {
                tmp = tmp.replace(&**from, to).into();
            }
            // pass on the trimmed string as well as how much we removed when replacing;
            // this is used to adjust column trimming in the scan operation, below.
            (tmp.to_string(), before - tmp.len())
        })
        .scan(
            false,
            |table_started, (line, trimmed)| match trim_path_col {
                // No trim configuration set, no further trimming needed
                None => Some(line),
                // Use trim configuration to trim Path columns
                Some((from, until)) => {
                    if !(*table_started || is_table(&line)) {
                        Some(line)
                    } else {
                        *table_started = true;
                        // Once we are inside a table, trim the path column by cutting out the characters
                        // at [from, until) (taking into account how much the line has already shrunk due
                        // to replacements)
                        let until = until - trimmed;
                        let chars = line.chars();
                        Some(chars.clone().take(from).chain(chars.skip(until)).collect())
                    }
                }
            },
        )
        .collect::<Vec<String>>()
        .join("\n")
}
