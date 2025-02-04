use std::collections::BTreeMap;

use comfy_table::{Attribute as ComfyAttribute, Cell, CellAlignment, Table};
use crossterm::style::Color;

use pueue_lib::network::message::{TaskLogMessage, TaskSelection};
use pueue_lib::settings::Settings;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use super::OutputStyle;
use crate::client::cli::SubCommand;
use crate::client::client::selection_from_params;

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
/// (unless otherwise explicitly requested).
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

/// Print the log output of finished tasks.
/// Either print the logs of every task
/// or only print the logs of the specified tasks.
pub fn print_logs(
    mut task_logs: BTreeMap<usize, TaskLogMessage>,
    cli_command: &SubCommand,
    style: &OutputStyle,
    settings: &Settings,
) {
    // Get actual commandline options.
    // This is necessary to know how we should display/return the log information.
    let SubCommand::Log {
        json,
        task_ids,
        group,
        lines,
        full,
        all,
    } = cli_command
    else {
        panic!("Got wrong Subcommand {cli_command:?} in print_log. This shouldn't happen");
    };

    let lines = determine_log_line_amount(*full, lines);

    // Return the server response in json representation.
    if *json {
        print_log_json(task_logs, settings, lines);
        return;
    }

    let selection = selection_from_params(*all, group.clone(), task_ids.clone());
    if task_logs.is_empty() {
        match selection {
            TaskSelection::TaskIds(_) => {
                eprintln!("There are no finished tasks for your specified ids");
                return;
            }
            TaskSelection::Group(group) => {
                eprintln!("There are no finished tasks for group '{group}'");
                return;
            }
            TaskSelection::All => {
                eprintln!("There are no finished tasks");
                return;
            }
        }
    }

    // Iterate over each task and print the respective log.
    let mut task_iter = task_logs.iter_mut().peekable();
    while let Some((_, task_log)) = task_iter.next() {
        print_log(task_log, style, settings, lines);

        // Add a newline if there is another task that's going to be printed.
        if let Some((_, task_log)) = task_iter.peek() {
            if matches!(
                &task_log.task.status,
                TaskStatus::Done { .. } | TaskStatus::Running { .. } | TaskStatus::Paused { .. }
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
    style: &OutputStyle,
    settings: &Settings,
    lines: Option<usize>,
) {
    let task = &message.task;
    // We only show logs of finished or running tasks.
    if !matches!(
        task.status,
        TaskStatus::Done { .. } | TaskStatus::Running { .. } | TaskStatus::Paused { .. }
    ) {
        return;
    }

    print_task_info(task, style);

    if settings.client.read_local_logs {
        print_local_log(message.task.id, style, settings, lines);
    } else if message.output.is_some() {
        print_remote_log(message, style, lines);
    } else {
        println!("Logs requested from pueue daemon, but none received. Please report this bug.");
    }
}

/// Print some information about a task, which is displayed on top of the task's log output.
fn print_task_info(task: &Task, style: &OutputStyle) {
    // Print task id and exit code.
    let task_cell = style.styled_cell(
        format!("Task {}: ", task.id),
        None,
        Some(ComfyAttribute::Bold),
    );

    let (exit_status, color) = match &task.status {
        TaskStatus::Paused { .. } => ("paused".into(), Color::White),
        TaskStatus::Running { .. } => ("running".into(), Color::Yellow),
        TaskStatus::Done { result, .. } => match result {
            TaskResult::Success => ("completed successfully".into(), Color::Green),
            TaskResult::Failed(exit_code) => {
                (format!("failed with exit code {exit_code}"), Color::Red)
            }
            TaskResult::FailedToSpawn(_err) => ("Failed to spawn".to_string(), Color::Red),
            TaskResult::Killed => ("killed by system or user".into(), Color::Red),
            TaskResult::Errored => ("some IO error.\n Check daemon log.".into(), Color::Red),
            TaskResult::DependencyFailed => ("dependency failed".into(), Color::Red),
        },
        _ => (task.status.to_string(), Color::White),
    };
    let status_cell = style.styled_cell(exit_status, Some(color), None);

    // The styling of the task number and status is done by a single-row table.
    let mut table = Table::new();
    table.load_preset("││─ └──┘     ─ ┌┐  ");
    table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
    table.set_header(vec![task_cell, status_cell]);

    // Explicitly force styling, in case we aren't on a tty, but `--color=always` is set.
    if style.enabled {
        table.enforce_styling();
    }
    eprintln!("{table}");

    // All other information is aligned and styled by using a separate table.
    let mut table = Table::new();
    table.load_preset(comfy_table::presets::NOTHING);
    table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

    // Command and path
    table.add_row(vec![
        style.styled_cell("Command:", None, Some(ComfyAttribute::Bold)),
        Cell::new(&task.command),
    ]);
    table.add_row(vec![
        style.styled_cell("Path:", None, Some(ComfyAttribute::Bold)),
        Cell::new(task.path.to_string_lossy()),
    ]);
    if let Some(label) = &task.label {
        table.add_row(vec![
            style.styled_cell("Label:", None, Some(ComfyAttribute::Bold)),
            Cell::new(label),
        ]);
    }

    let (start, end) = task.start_and_end();

    // Start and end time
    if let Some(start) = start {
        table.add_row(vec![
            style.styled_cell("Start:", None, Some(ComfyAttribute::Bold)),
            Cell::new(start.to_rfc2822()),
        ]);
    }
    if let Some(end) = end {
        table.add_row(vec![
            style.styled_cell("End:", None, Some(ComfyAttribute::Bold)),
            Cell::new(end.to_rfc2822()),
        ]);
    }

    // Set the padding of the left column to 0 align the keys to the right
    let first_column = table.column_mut(0).unwrap();
    first_column.set_cell_alignment(CellAlignment::Right);
    first_column.set_padding((0, 0));

    eprintln!("{table}");
}
