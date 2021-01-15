use std::{collections::BTreeMap, io::BufReader};
use std::{
    fs::File,
    io::{self, Stdout},
};

use anyhow::Result;
use comfy_table::*;
use snap::read::FrameDecoder;

use pueue::log::get_log_file_handles;
use pueue::network::message::TaskLogMessage;
use pueue::settings::Settings;
use pueue::task::{TaskResult, TaskStatus};

use super::helper::*;
use crate::cli::SubCommand;

/// Print the log ouput of finished tasks.
/// Either print the logs of every task
/// or only print the logs of the specified tasks.
pub fn print_logs(
    mut task_logs: BTreeMap<usize, TaskLogMessage>,
    cli_command: &SubCommand,
    settings: &Settings,
) {
    // Get actual commandline options.
    // This is necessary to know how we should display/return the log information.
    let (json, task_ids, lines, full) = match cli_command {
        SubCommand::Log {
            json,
            task_ids,
            lines,
            full,
        } => (*json, task_ids.clone(), lines.clone(), *full),
        _ => panic!(
            "Got wrong Subcommand {:?} in print_log. This shouldn't happen",
            cli_command
        ),
    };

    // Return the server response in json representation.
    // TODO: This only works properly if we get the logs from remote.
    // TODO: However, this still doesn't work, since the logs are still compressed.
    if json {
        println!("{}", serde_json::to_string(&task_logs).unwrap());
        return;
    }

    // Check some early return conditions
    if task_ids.is_empty() && task_logs.is_empty() {
        println!("There are no finished tasks");
        return;
    }

    if !task_ids.is_empty() && task_logs.is_empty() {
        println!("There are no finished tasks for your specified ids");
        return;
    }

    // Determine, whether we should draw everything or only a part of the log output.
    // None implicates that all lines are printed
    let lines = if full {
        None
    } else if let Some(lines) = lines {
        Some(lines)
    } else {
        // By default only some lines are shown per task, if multiple tasks exist.
        // For a single task, the whole log output is shown.
        if task_logs.len() > 1 {
            Some(15)
        } else {
            None
        }
    };

    // Do the actual log printing
    let mut task_iter = task_logs.iter_mut().peekable();
    while let Some((_, mut task_log)) = task_iter.next() {
        print_log(&mut task_log, settings, lines);

        // Add a newline if there is another task that's going to be printed.
        if let Some((_, task_log)) = task_iter.peek() {
            if vec![TaskStatus::Done, TaskStatus::Running, TaskStatus::Paused]
                .contains(&task_log.task.status)
            {
                println!("\n");
            }
        }
    }
}

/// Print the log of a single task.
///
/// message: The message returned by the daemon. This message includes all
///          requested tasks and the tasks' logs, if we don't read local logs.
/// lines: Whether we should reduce the log output of each task to a specific number of lines.
///         `None` implicates that everything should be printed.
///         This is only important, if we read local lines.
pub fn print_log(message: &mut TaskLogMessage, settings: &Settings, lines: Option<usize>) {
    let task = &message.task;
    // We only show logs of finished or running tasks.
    if !vec![TaskStatus::Done, TaskStatus::Running, TaskStatus::Paused].contains(&task.status) {
        return;
    }

    // Print task id and exit code.
    let task_text = style_text(&format!("Task {}", task.id), None, Some(Attribute::Bold));
    let (exit_status, color) = match &task.result {
        Some(TaskResult::Success) => ("completed successfully".into(), Color::Green),
        Some(TaskResult::Failed(exit_code)) => {
            (format!("failed with exit code {}", exit_code), Color::Red)
        }
        Some(TaskResult::FailedToSpawn(err)) => (format!("failed to spawn: {}", err), Color::Red),
        Some(TaskResult::Killed) => ("killed by system or user".into(), Color::Red),
        Some(TaskResult::Errored) => ("some IO error.\n Check daemon log.".into(), Color::Red),
        Some(TaskResult::DependencyFailed) => ("dependency failed".into(), Color::Red),
        None => ("running".into(), Color::White),
    };
    let status_text = style_text(&exit_status, Some(color), None);
    println!("{} {}", task_text, status_text);

    // Print command and path.
    println!("Command: {}", task.command);
    println!("Path: {}", task.path);

    if let Some(start) = task.start {
        println!("Start: {}", start.to_rfc2822());
    }
    if let Some(end) = task.end {
        println!("End: {}", end.to_rfc2822());
    }

    if settings.client.read_local_logs {
        print_local_log(message.task.id, settings, lines);
    } else if message.stdout.is_some() && message.stderr.is_some() {
        print_remote_log(message);
    } else {
        println!("Logs requested from pueue daemon, but none received. Please report this bug.");
    }
}

/// The daemon didn't send any log output, thereby we didn't request any.
/// If that's the case, read the log files from the local pueue directory
pub fn print_local_log(task_id: usize, settings: &Settings, lines: Option<usize>) {
    let (mut stdout_log, mut stderr_log) =
        match get_log_file_handles(task_id, &settings.shared.pueue_directory) {
            Ok((stdout, stderr)) => (stdout, stderr),
            Err(err) => {
                println!("Failed to get log file handles: {}", err);
                return;
            }
        };
    // Stdout handler to directly write log file output to io::stdout
    // without having to load anything into memory.
    let mut stdout = io::stdout();

    print_local_file(
        &mut stdout,
        &mut stdout_log,
        &lines,
        style_text("stdout:", Some(Color::Green), Some(Attribute::Bold)),
    );

    print_local_file(
        &mut stdout,
        &mut stderr_log,
        &lines,
        style_text("stderr:", Some(Color::Red), Some(Attribute::Bold)),
    );
}

/// Print a local log file.
/// This is usually either the stdout or the stderr
pub fn print_local_file(stdout: &mut Stdout, file: &mut File, lines: &Option<usize>, text: String) {
    if let Ok(metadata) = file.metadata() {
        if metadata.len() != 0 {
            println!("\n{}", text);
            // Only print the last lines if requested
            if let Some(lines) = lines {
                // TODO: This is super imperformant, but works as long as we don't use the last
                // 1000 lines. It would be cleaner to seek to the beginning of the requested
                // position and simply stream the content to stdout.
                let last_lines: Vec<String> = rev_lines::RevLines::new(BufReader::new(file))
                    .expect("Failed to read last lines of file")
                    .take(*lines)
                    .collect();

                println!(
                    "{}",
                    last_lines
                        .into_iter()
                        .rev()
                        .collect::<Vec<String>>()
                        .join("\n")
                );
                return;
            }

            // Print everything
            if let Err(err) = io::copy(file, stdout) {
                println!("Failed reading local log file: {}", err);
            };
        }
    }
}

/// Prints log output received from the daemon.
/// We can safely call .unwrap() on stdout and stderr in here, since this
/// branch is always called after ensuring that both are `Some`.
pub fn print_remote_log(task_log: &TaskLogMessage) {
    // Save whether stdout was printed, so we can add a newline between outputs.
    if !task_log.stdout.as_ref().unwrap().is_empty() {
        if let Err(err) = print_remote_task_log(&task_log, true) {
            println!("Error while parsing stdout: {}", err);
        }
    }

    if !task_log.stderr.as_ref().unwrap().is_empty() {
        if let Err(err) = print_remote_task_log(&task_log, false) {
            println!("Error while parsing stderr: {}", err);
        };
    }
}

/// Print log output of a finished process.
pub fn print_remote_task_log(task_log: &TaskLogMessage, stdout: bool) -> Result<()> {
    let (pre_text, color, bytes) = if stdout {
        ("stdout: ", Color::Green, task_log.stdout.as_ref().unwrap())
    } else {
        ("stderr: ", Color::Red, task_log.stderr.as_ref().unwrap())
    };

    println!(
        "\n{}",
        style_text(pre_text, Some(color), Some(Attribute::Bold))
    );

    let mut decompressor = FrameDecoder::new(bytes.as_slice());

    let stdout = io::stdout();
    let mut write = stdout.lock();
    io::copy(&mut decompressor, &mut write)?;

    Ok(())
}
