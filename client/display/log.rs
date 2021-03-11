use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Stdout};

use anyhow::Result;
use comfy_table::*;
use snap::read::FrameDecoder;

use pueue_lib::log::{get_log_file_handles, read_last_lines};
use pueue_lib::network::message::TaskLogMessage;
use pueue_lib::settings::Settings;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use super::{colors::Colors, helper::*};
use crate::cli::SubCommand;

/// Print the log ouput of finished tasks.
/// Either print the logs of every task
/// or only print the logs of the specified tasks.
pub fn print_logs(
    mut task_logs: BTreeMap<usize, TaskLogMessage>,
    cli_command: &SubCommand,
    colors: &Colors,
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
        } => (*json, task_ids.clone(), *lines, *full),
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
        print_log(&mut task_log, colors, settings, lines);

        // Add a newline if there is another task that's going to be printed.
        if let Some((_, task_log)) = task_iter.peek() {
            if vec![TaskStatus::Done, TaskStatus::Running, TaskStatus::Paused]
                .contains(&task_log.task.status)
            {
                println!();
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
pub fn print_log(
    message: &mut TaskLogMessage,
    colors: &Colors,
    settings: &Settings,
    lines: Option<usize>,
) {
    let task = &message.task;
    // We only show logs of finished or running tasks.
    if !vec![TaskStatus::Done, TaskStatus::Running, TaskStatus::Paused].contains(&task.status) {
        return;
    }

    print_task_info(task, colors);

    if settings.client.read_local_logs {
        print_local_log(message.task.id, colors, settings, lines);
    } else if message.stdout.is_some() && message.stderr.is_some() {
        print_remote_log(message, colors);
    } else {
        println!("Logs requested from pueue daemon, but none received. Please report this bug.");
    }
}

/// Print some information about a task, which is displayed on top of the task's log output.
fn print_task_info(task: &Task, colors: &Colors) {
    // Print task id and exit code.
    let task_cell = Cell::new(format!("Task {}: ", task.id)).add_attribute(Attribute::Bold);

    let (exit_status, color) = match &task.result {
        Some(TaskResult::Success) => ("completed successfully".into(), colors.green()),
        Some(TaskResult::Failed(exit_code)) => {
            (format!("failed with exit code {}", exit_code), colors.red())
        }
        Some(TaskResult::FailedToSpawn(err)) => (format!("failed to spawn: {}", err), colors.red()),
        Some(TaskResult::Killed) => ("killed by system or user".into(), colors.red()),
        Some(TaskResult::Errored) => ("some IO error.\n Check daemon log.".into(), colors.red()),
        Some(TaskResult::DependencyFailed) => ("dependency failed".into(), colors.red()),
        None => match &task.status {
            TaskStatus::Paused => ("paused".into(), colors.white()),
            TaskStatus::Running => ("running".into(), colors.yellow()),
            _ => (task.status.to_string(), colors.white()),
        },
    };
    let status_cell = Cell::new(exit_status).fg(color);

    // The styling of the task number and status is done by a single-row table.
    let mut table = Table::new();
    table.load_preset("││─ └──┘     ─ ┌┐  ");
    table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
    table.set_header(vec![task_cell, status_cell]);
    println!("{}", table);

    // All other information is alligned and styled by using a separat table.
    let mut table = Table::new();
    table.load_preset(comfy_table::presets::NOTHING);
    table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

    // Command and path
    table.add_row(vec![
        Cell::new("Command:").add_attribute(Attribute::Bold),
        Cell::new(&task.command),
    ]);
    table.add_row(vec![
        Cell::new("Path:").add_attribute(Attribute::Bold),
        Cell::new(&task.path),
    ]);

    // Start and end time
    if let Some(start) = task.start {
        table.add_row(vec![
            Cell::new("Start:").add_attribute(Attribute::Bold),
            Cell::new(start.to_rfc2822()),
        ]);
    }
    if let Some(end) = task.end {
        table.add_row(vec![
            Cell::new("End:").add_attribute(Attribute::Bold),
            Cell::new(end.to_rfc2822()),
        ]);
    }

    // Set the padding of the left column to 0 align the keys to the right
    let first_column = table.get_column_mut(0).unwrap();
    first_column.set_cell_alignment(CellAlignment::Right);
    first_column.set_padding((0, 0));

    println!("{}", table);
}

/// The daemon didn't send any log output, thereby we didn't request any.
/// If that's the case, read the log files from the local pueue directory
fn print_local_log(task_id: usize, colors: &Colors, settings: &Settings, lines: Option<usize>) {
    let (mut stdout_file, mut stderr_file) =
        match get_log_file_handles(task_id, &settings.shared.pueue_directory()) {
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
        &mut stdout_file,
        &lines,
        style_text("stdout:", Some(colors.green()), Some(Attribute::Bold)),
    );

    print_local_file(
        &mut stdout,
        &mut stderr_file,
        &lines,
        style_text("stderr:", Some(colors.red()), Some(Attribute::Bold)),
    );
}

/// Print a local log file.
/// This is usually either the stdout or the stderr
pub fn print_local_file(stdout: &mut Stdout, file: &mut File, lines: &Option<usize>, text: String) {
    if let Ok(metadata) = file.metadata() {
        if metadata.len() != 0 {
            // Don't print a newline between the task information and the first output
            println!("\n{}", text);

            // Only print the last lines if requested
            if let Some(lines) = lines {
                println!("{}", read_last_lines(file, *lines));
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
pub fn print_remote_log(task_log: &TaskLogMessage, colors: &Colors) {
    // Save whether stdout was printed, so we can add a newline between outputs.
    if let Some(bytes) = task_log.stdout.as_ref() {
        if !bytes.is_empty() {
            println!(
                "\n{}",
                style_text("stdout: ", Some(colors.green()), Some(Attribute::Bold))
            );

            if let Err(err) = decompress_and_print_remote_log(bytes) {
                println!("Error while parsing stdout: {}", err);
            }
        }
    }

    if let Some(bytes) = task_log.stderr.as_ref() {
        if !bytes.is_empty() {
            println!(
                "\n{}",
                style_text("stderr: ", Some(colors.red()), Some(Attribute::Bold))
            );

            if let Err(err) = decompress_and_print_remote_log(bytes) {
                println!("Error while parsing stderr: {}", err);
            };
        }
    }
}

/// We cannot easily stream log output from the client to the daemon (yet).
/// Right now, stdout and stderr are compressed in the daemon and sent as a single payload to the
/// client. In here, we take that payload, decompress it and stream it it directly to stdout.
fn decompress_and_print_remote_log(bytes: &[u8]) -> Result<()> {
    let mut decompressor = FrameDecoder::new(bytes);

    let stdout = io::stdout();
    let mut write = stdout.lock();
    io::copy(&mut decompressor, &mut write)?;

    Ok(())
}
