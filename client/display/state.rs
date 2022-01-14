use std::collections::BTreeMap;
use std::string::ToString;

use chrono::{Duration, Local};
use comfy_table::presets::UTF8_HORIZONTAL_ONLY;
use comfy_table::*;

use pueue_lib::settings::Settings;
use pueue_lib::state::{State, PUEUE_DEFAULT_GROUP};
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use super::{colors::Colors, helper::*};
use crate::cli::SubCommand;

/// Print the current state of the daemon in a nicely formatted table.
pub fn print_state(state: State, cli_command: &SubCommand, colors: &Colors, settings: &Settings) {
    let (json, group_only) = match cli_command {
        SubCommand::Status { json, group } => (*json, group.clone()),
        SubCommand::FormatStatus { group } => (false, group.clone()),
        _ => panic!("Got wrong Subcommand {cli_command:?} in print_state. This shouldn't happen!"),
    };

    // If the json flag is specified, print the state as json and exit.
    if json {
        println!("{}", serde_json::to_string(&state).unwrap());
        return;
    }

    // Sort all tasks by their respective group;
    let sorted_tasks = sort_tasks_by_group(&state.tasks);

    if let Some(group) = group_only {
        print_single_group(state, settings, colors, sorted_tasks, group);
        return;
    }

    print_all_groups(state, settings, colors, sorted_tasks);
}

fn print_single_group(
    state: State,
    settings: &Settings,
    colors: &Colors,
    mut sorted_tasks: BTreeMap<String, BTreeMap<usize, Task>>,
    group_name: String,
) {
    let group = if let Some(group) = state.groups.get(&group_name) {
        group
    } else {
        eprintln!("There exists no group \"{group_name}\"");
        return;
    };

    // Only a single group is requested. Print that group and return.
    let tasks = sorted_tasks.entry(group_name.clone()).or_default();
    let headline = get_group_headline(&group_name, group, colors);
    println!("{headline}");

    // Show a message if the requested group doesn't have any tasks.
    if tasks.is_empty() {
        println!("Task list is empty. Add tasks with `pueue add -g {group_name} -- [cmd]`");
        return;
    }
    print_table(tasks, colors, settings);
}

fn print_all_groups(
    state: State,
    settings: &Settings,
    colors: &Colors,
    sorted_tasks: BTreeMap<String, BTreeMap<usize, Task>>,
) {
    // Early exit and hint if there are no tasks in the queue
    // Print the state of the default group anyway, since this is information one wants to
    // see most of the time anyway.
    if state.tasks.is_empty() {
        let headline = get_group_headline(
            PUEUE_DEFAULT_GROUP,
            state.groups.get(PUEUE_DEFAULT_GROUP).unwrap(),
            colors,
        );
        println!("{headline}\n");
        println!("Task list is empty. Add tasks with `pueue add -- [cmd]`");
        return;
    }

    // Always print the default queue at the very top, if no specific group is requested.
    if sorted_tasks.get(PUEUE_DEFAULT_GROUP).is_some() {
        let tasks = sorted_tasks.get(PUEUE_DEFAULT_GROUP).unwrap();
        let headline = get_group_headline(
            PUEUE_DEFAULT_GROUP,
            state.groups.get(PUEUE_DEFAULT_GROUP).unwrap(),
            colors,
        );
        println!("{headline}");
        print_table(tasks, colors, settings);

        // Add a newline if there are further groups to be printed
        if sorted_tasks.len() > 1 {
            println!();
        }
    }

    // Print a table for every other group that has any tasks
    let mut sorted_iter = sorted_tasks.iter().peekable();
    while let Some((group, tasks)) = sorted_iter.next() {
        // We always want to print the default group at the very top.
        // That's why we print it before this loop and skip it in here.
        if group.eq(PUEUE_DEFAULT_GROUP) {
            continue;
        }

        let headline = get_group_headline(group, state.groups.get(group).unwrap(), colors);
        println!("{headline}");
        print_table(tasks, colors, settings);

        // Add a newline between groups
        if sorted_iter.peek().is_some() {
            println!();
        }
    }
}

/// Print some tasks into a nicely formatted table
fn print_table(tasks: &BTreeMap<usize, Task>, colors: &Colors, settings: &Settings) {
    let (has_delayed_tasks, has_dependencies, has_labels) = has_special_columns(tasks);

    // Create table header row
    let mut headers = vec![Cell::new("Id"), Cell::new("Status")];

    if has_delayed_tasks {
        headers.push(Cell::new("Enqueue At"));
    }
    if has_dependencies {
        headers.push(Cell::new("Deps"));
    }
    if has_labels {
        headers.push(Cell::new("Label"));
    }

    headers.append(&mut vec![
        Cell::new("Command"),
        Cell::new("Path"),
        Cell::new("Start"),
        Cell::new("End"),
    ]);

    // Initialize comfy table.
    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .load_preset(UTF8_HORIZONTAL_ONLY)
        .set_header(headers);

    // Add rows one by one.
    for (id, task) in tasks {
        let mut row = Row::new();
        if let Some(height) = settings.client.max_status_lines {
            row.max_height(height);
        }
        row.add_cell(Cell::new(&id.to_string()));

        // Determine the human readable task status representation and the respective color.
        let status_string = task.status.to_string();
        let (status_text, color) = match &task.status {
            TaskStatus::Running => (status_string, colors.green()),
            TaskStatus::Paused | TaskStatus::Locked => (status_string, colors.white()),
            TaskStatus::Done(result) => match result {
                TaskResult::Success => (TaskResult::Success.to_string(), colors.green()),
                TaskResult::DependencyFailed => ("Dependency failed".to_string(), colors.red()),
                TaskResult::FailedToSpawn(_) => ("Failed to spawn".to_string(), colors.red()),
                TaskResult::Failed(code) => (format!("Failed ({code})"), colors.red()),
                _ => (result.to_string(), colors.red()),
            },
            _ => (status_string, colors.yellow()),
        };
        row.add_cell(Cell::new(status_text).fg(color));

        if has_delayed_tasks {
            if let TaskStatus::Stashed {
                enqueue_at: Some(enqueue_at),
            } = task.status
            {
                // Only show the date if the task is supposed to be enqueued today.
                let enqueue_today =
                    enqueue_at <= Local::today().and_hms(0, 0, 0) + Duration::days(1);
                let formatted_enqueue_at = if enqueue_today {
                    enqueue_at.format(&settings.client.status_time_format)
                } else {
                    enqueue_at.format(&settings.client.status_datetime_format)
                };
                row.add_cell(Cell::new(formatted_enqueue_at));
            } else {
                row.add_cell(Cell::new(""));
            }
        }

        if has_dependencies {
            let text = task
                .dependencies
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            row.add_cell(Cell::new(text));
        }

        if has_labels {
            if let Some(label) = &task.label {
                row.add_cell(label.into());
            } else {
                row.add_cell(Cell::new(""));
            }
        }

        // Add command and path.
        if settings.client.show_expanded_aliases {
            row.add_cell(Cell::new(&task.command));
        } else {
            row.add_cell(Cell::new(&task.original_command));
        }
        row.add_cell(Cell::new(&task.path));

        // Add start and end info
        let (start, end) = formatted_start_end(task, settings);
        row.add_cell(Cell::new(start));
        row.add_cell(Cell::new(end));

        table.add_row(row);
    }

    // Print the table.
    println!("{table}");
}

/// Returns the formatted `start` and `end` text for a given task.
///
/// 1. If the start || end is today, skip the date.
/// 2. Otherwise show the date in both.
///
/// If the task doesn't have a start and/or end yet, an empty string will be returned
/// for the respective field.
fn formatted_start_end(task: &Task, settings: &Settings) -> (String, String) {
    // Get the start time.
    // If the task didn't start yet, just return two empty strings.
    let start = match task.start {
        Some(start) => start,
        None => return ("".into(), "".into()),
    };

    // If the task started today, just show the time.
    // Otherwise show the full date and time.
    let started_today = start >= Local::today().and_hms(0, 0, 0);
    let formatted_start = if started_today {
        start
            .format(&settings.client.status_time_format)
            .to_string()
    } else {
        start
            .format(&settings.client.status_datetime_format)
            .to_string()
    };

    // Get finish time, if already set. Otherwise only return the formatted start.
    let end = match task.end {
        Some(end) => end,
        None => return (formatted_start, "".into()),
    };

    // If the task ended today we only show the time.
    // In all other circumstances, we show the full date.
    let finished_today = end >= Local::today().and_hms(0, 0, 0);
    let formatted_end = if finished_today {
        end.format(&settings.client.status_time_format).to_string()
    } else {
        end.format(&settings.client.status_datetime_format)
            .to_string()
    };

    (formatted_start, formatted_end)
}
