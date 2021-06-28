use std::collections::BTreeMap;
use std::string::ToString;

use comfy_table::presets::UTF8_HORIZONTAL_BORDERS_ONLY;
use comfy_table::*;

use pueue_lib::settings::Settings;
use pueue_lib::state::State;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use super::{colors::Colors, helper::*};
use crate::cli::SubCommand;

/// Print the current state of the daemon in a nicely formatted table.
pub fn print_state(state: State, cli_command: &SubCommand, colors: &Colors, settings: &Settings) {
    let (json, group_only) = match cli_command {
        SubCommand::Status { json, group } => (*json, group.clone()),
        _ => panic!(
            "Got wrong Subcommand {:?} in print_state. This shouldn't happen",
            cli_command
        ),
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
    group: String,
) {
    // Only a single group is requested. Print that group and return.
    let tasks = sorted_tasks.entry(group.clone()).or_default();
    let headline = get_group_headline(
        &group,
        state.groups.get(&group).unwrap(),
        *state.settings.daemon.groups.get(&group).unwrap(),
        colors,
    );
    println!("{}", headline);

    // Show a message if the requested group doesn't have any tasks.
    if tasks.is_empty() {
        println!(
            "Task list is empty. Add tasks with `pueue add -g {} -- [cmd]`",
            group
        );
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
            "default",
            state.groups.get("default").unwrap(),
            *state.settings.daemon.groups.get("default").unwrap(),
            colors,
        );
        println!("{}\n", headline);
        println!("Task list is empty. Add tasks with `pueue add -- [cmd]`");
        return;
    }

    // Always print the default queue at the very top, if no specific group is requested.
    if sorted_tasks.get("default").is_some() {
        let tasks = sorted_tasks.get("default").unwrap();
        let headline = get_group_headline(
            "default",
            state.groups.get("default").unwrap(),
            *state.settings.daemon.groups.get("default").unwrap(),
            colors,
        );
        println!("{}", headline);
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
        if group.eq("default") {
            continue;
        }

        let headline = get_group_headline(
            group,
            state.groups.get(group).unwrap(),
            *state.settings.daemon.groups.get(group).unwrap(),
            colors,
        );
        println!("{}", headline);
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
    let mut headers = vec![Cell::new("Index"), Cell::new("Status")];
    if has_delayed_tasks {
        headers.push(Cell::new("Enqueue At"));
    }
    if has_dependencies {
        headers.push(Cell::new("Deps"));
    }

    headers.push(Cell::new("Exitcode"));

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
        .load_preset(UTF8_HORIZONTAL_BORDERS_ONLY)
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
                row.add_cell(Cell::new(enqueue_at.format("%Y-%m-%d\n%H:%M:%S")));
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

        // Match the color of the exit code.
        // If the exit_code is none, it has been killed by the task handler.
        let exit_code_cell = match task.status {
            TaskStatus::Done(TaskResult::Success) => Cell::new("0").fg(colors.green()),
            TaskStatus::Done(TaskResult::Failed(code)) => {
                Cell::new(&code.to_string()).fg(colors.red())
            }
            _ => Cell::new(""),
        };
        row.add_cell(exit_code_cell);
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

        // Add start time, if already set.
        if let Some(start) = task.start {
            let formatted = start.format("%H:%M").to_string();
            row.add_cell(Cell::new(&formatted));
        } else {
            row.add_cell(Cell::new(""));
        }

        // Add finish time, if already set.
        if let Some(end) = task.end {
            let formatted = end.format("%H:%M").to_string();
            row.add_cell(Cell::new(&formatted));
        } else {
            row.add_cell(Cell::new(""));
        }

        table.add_row(row);
    }

    // Print the table.
    println!("{}", table);
}
