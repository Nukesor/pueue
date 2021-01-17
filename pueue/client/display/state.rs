use std::collections::BTreeMap;
use std::string::ToString;

use comfy_table::presets::UTF8_HORIZONTAL_BORDERS_ONLY;
use comfy_table::*;

use pueue_lib::settings::Settings;
use pueue_lib::state::State;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use super::helper::*;
use crate::cli::SubCommand;

/// Print the current state of the daemon in a nicely formatted table.
pub fn print_state(state: State, cli_command: &SubCommand, settings: &Settings) {
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

    // Early exit and hint if there are no tasks in the queue
    if state.tasks.is_empty() {
        println!("Task list is empty. Add tasks with `pueue add -- [cmd]`");
        return;
    }

    // Sort all tasks by their respective group;
    let sorted_tasks = sort_tasks_by_group(&state.tasks);

    // Always print the default queue at the very top.
    if group_only.is_none() && sorted_tasks.get("default").is_some() {
        let tasks = sorted_tasks.get("default").unwrap();
        let headline = get_group_headline(
            &"default",
            &state.groups.get("default").unwrap(),
            *state.settings.daemon.groups.get("default").unwrap(),
        );
        println!("{}", headline);
        print_table(&tasks, settings);

        // Add a newline if there are further groups to be printed
        if sorted_tasks.len() > 1 {
            println!();
        }
    }

    let mut sorted_iter = sorted_tasks.iter().peekable();
    // Print new table for each group
    while let Some((group, tasks)) = sorted_iter.next() {
        // We always want to print the default group at the very top.
        // That's why we print it outside of this loop and skip it in here.
        if group.eq("default") {
            continue;
        }

        // Skip unwanted groups, if a single group is requested
        if let Some(group_only) = &group_only {
            if group_only != group {
                continue;
            }
        }
        let headline = get_group_headline(
            &group,
            &state.groups.get(group).unwrap(),
            *state.settings.daemon.groups.get(group).unwrap(),
        );
        println!("{}", headline);
        print_table(&tasks, settings);

        // Add a newline between groups
        if sorted_iter.peek().is_some() {
            println!();
        }
    }
}

/// Print some tasks into a nicely formatted table
fn print_table(tasks: &BTreeMap<usize, Task>, settings: &Settings) {
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
        let (status_text, color) = match task.status {
            TaskStatus::Running => (status_string, Color::Green),
            TaskStatus::Paused | TaskStatus::Locked => (status_string, Color::White),
            TaskStatus::Done => match &task.result {
                Some(TaskResult::Success) => (TaskResult::Success.to_string(), Color::Green),
                Some(TaskResult::DependencyFailed) => ("Dependency failed".to_string(), Color::Red),
                Some(TaskResult::FailedToSpawn(_)) => ("Failed to spawn".to_string(), Color::Red),
                Some(result) => (result.to_string(), Color::Red),
                None => panic!("Got a 'Done' task without a task result. Please report this bug."),
            },
            _ => (status_string, Color::Yellow),
        };
        row.add_cell(Cell::new(status_text).fg(color));

        if has_delayed_tasks {
            if let Some(enqueue_at) = task.enqueue_at {
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
        let exit_code_cell = match task.result {
            Some(TaskResult::Success) => Cell::new("0").fg(Color::Green),
            Some(TaskResult::Failed(code)) => Cell::new(&code.to_string()).fg(Color::Red),
            _ => Cell::new(""),
        };
        row.add_cell(exit_code_cell);
        if has_labels {
            if let Some(label) = &task.label {
                row.add_cell(label.to_cell());
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
