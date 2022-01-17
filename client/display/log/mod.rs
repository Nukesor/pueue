use std::collections::BTreeMap;

use comfy_table::*;

use pueue_lib::network::message::TaskLogMessage;
use pueue_lib::settings::Settings;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use super::colors::Colors;
use crate::cli::SubCommand;

mod json;
mod local;
mod remote;

use json::*;
use local::*;
use remote::*;

/// Determine how many lines of output should be printed/returned.
/// `None` implicates that all lines are printed.
///
/// By default, everything is returned for single tasks and only some lines for multiple.
/// `json` is an exception to this, in json mode we always only return some lines
/// (unless otherwise explicitely requested).
///
/// `full` always forces the full log output
/// `lines` force a specific amount of lines
pub fn determine_log_line_amount(full: bool, lines: &Option<usize>) -> Option<usize> {
    if full {
        None
    } else if let Some(lines) = lines {
        Some(*lines)
    } else {
        // By default, only some lines are shown per task
        Some(15)
    }
}

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
        _ => panic!("Got wrong Subcommand {cli_command:?} in print_log. This shouldn't happen"),
    };

    let lines = determine_log_line_amount(full, &lines);

    // Return the server response in json representation.
    if json {
        print_log_json(task_logs, settings, lines);
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

    // Iterate over each task and print the respective log.
    let mut task_iter = task_logs.iter_mut().peekable();
    while let Some((_, task_log)) = task_iter.next() {
        print_log(task_log, colors, settings, lines);

        // Add a newline if there is another task that's going to be printed.
        if let Some((_, task_log)) = task_iter.peek() {
            if matches!(
                &task_log.task.status,
                TaskStatus::Done(_) | TaskStatus::Running | TaskStatus::Paused,
            ) {
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
fn print_log(
    message: &mut TaskLogMessage,
    colors: &Colors,
    settings: &Settings,
    lines: Option<usize>,
) {
    let task = &message.task;
    // We only show logs of finished or running tasks.
    if !matches!(
        task.status,
        TaskStatus::Done(_) | TaskStatus::Running | TaskStatus::Paused
    ) {
        return;
    }

    print_task_info(task, colors);

    if settings.client.read_local_logs {
        print_local_log(message.task.id, colors, settings, lines);
    } else if message.output.is_some() {
        print_remote_log(message, colors);
    } else {
        println!("Logs requested from pueue daemon, but none received. Please report this bug.");
    }
}

/// Print some information about a task, which is displayed on top of the task's log output.
fn print_task_info(task: &Task, colors: &Colors) {
    // Print task id and exit code.
    let task_cell = Cell::new(format!("Task {}: ", task.id)).add_attribute(Attribute::Bold);

    let (exit_status, color) = match &task.status {
        TaskStatus::Paused => ("paused".into(), colors.white()),
        TaskStatus::Running => ("running".into(), colors.yellow()),
        TaskStatus::Done(result) => match result {
            TaskResult::Success => ("completed successfully".into(), colors.green()),
            TaskResult::Failed(exit_code) => {
                (format!("failed with exit code {}", exit_code), colors.red())
            }
            TaskResult::FailedToSpawn(err) => (format!("failed to spawn: {}", err), colors.red()),
            TaskResult::Killed => ("killed by system or user".into(), colors.red()),
            TaskResult::Errored => ("some IO error.\n Check daemon log.".into(), colors.red()),
            TaskResult::DependencyFailed => ("dependency failed".into(), colors.red()),
        },
        _ => (task.status.to_string(), colors.white()),
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

    println!("{table}");
}
