use ::comfy_table::presets::UTF8_HORIZONTAL_BORDERS_ONLY;
use ::comfy_table::*;
use ::crossterm::style::style;
use ::std::collections::BTreeMap;
use ::std::string::ToString;

use ::pueue::state::State;
use ::pueue::task::{Task, TaskStatus};

use crate::cli::SubCommand;

pub fn print_success(message: String) {
    println!("{}", message);
}

pub fn print_error(message: String) {
    let styled = style(message).with(Color::Red);
    println!("{}", styled);
}

/// Print the current state of the daemon in a nicely formatted table
pub fn print_state(state: State, cli_command: &SubCommand) {
    let json = match cli_command {
        SubCommand::Status { json } => *json,
        _ => panic!(
            "Got wrong Subcommand {:?} in print_state. This shouldn't happen",
            cli_command
        ),
    };

    // If the json flag is specified, print the state as json and exit
    if json {
        println!("{}", serde_json::to_string(&state).unwrap());
        return;
    }

    // Print the current daemon state
    if state.running {
        println!("{}", style("Daemon status: running").with(Color::Green));
    } else {
        println!("{}", style("Daemon status: paused").with(Color::Yellow));
    };

    // Early exit and hint if there are no tasks in the queue
    if state.tasks.is_empty() {
        println!("\nTask list is empty. Add tasks with `pueue add -- [cmd]`");
        return;
    }

    // Check whether there are any delayed tasks.
    // In case there are, we need to add another column to the table
    let has_delayed_tasks = state
        .tasks
        .iter()
        .any(|(_id, task)| task.enqueue_at.is_some());

    // Check whether there are any tasks with dependencies.
    // In case there are, we need to add another column to the table
    let has_dependencies = state
        .tasks
        .iter()
        .any(|(_id, task)| !task.dependencies.is_empty());

    // Create table header row
    let mut headers = vec![Cell::new("Index"), Cell::new("Status")];
    if has_delayed_tasks {
        headers.push(Cell::new("Enqueue At"));
    }
    if has_dependencies {
        headers.push(Cell::new("Deps"));
    }
    headers.append(&mut vec![
        Cell::new("Exitcode"),
        Cell::new("Command"),
        Cell::new("Path"),
        Cell::new("Start"),
        Cell::new("End"),
    ]);

    // Initialize comfy table
    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .load_preset(UTF8_HORIZONTAL_BORDERS_ONLY)
        .set_header(headers);

    // Add rows one by one
    for (id, task) in state.tasks {
        let mut row = Row::new();
        row.add_cell(Cell::new(&id.to_string()));

        // Add status cell and color depending on state
        let status_cell = Cell::new(&task.status.to_string());
        let status_cell = match task.status {
            TaskStatus::Running | TaskStatus::Done => status_cell.fg(Color::Green),
            TaskStatus::Failed | TaskStatus::Killed => status_cell.fg(Color::Red),
            TaskStatus::Paused => status_cell.fg(Color::White),
            _ => status_cell.fg(Color::Yellow),
        };
        row.add_cell(status_cell);

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

        // Match the color of the exit code
        // If the exit_code is none, it has been killed by the task handler.
        match task.exit_code {
            Some(code) => {
                // Everything that's not 0, is failed task
                if code == 0 {
                    row.add_cell(Cell::new(&code.to_string()).fg(Color::Green));
                } else {
                    row.add_cell(Cell::new(&code.to_string()).fg(Color::Red));
                }
            }
            None => {
                row.add_cell(Cell::new(""));
            }
        }

        // Add command and path
        row.add_cell(Cell::new(&task.command));
        row.add_cell(Cell::new(&task.path));

        // Add start time, if already set
        if let Some(start) = task.start {
            let formatted = start.format("%H:%M").to_string();
            row.add_cell(Cell::new(&formatted));
        } else {
            row.add_cell(Cell::new(""));
        }

        // Add finish time, if already set
        if let Some(end) = task.end {
            let formatted = end.format("%H:%M").to_string();
            row.add_cell(Cell::new(&formatted));
        } else {
            row.add_cell(Cell::new(""));
        }

        table.add_row(row);
    }

    // Print the table
    println!("{}", table);
}

/// Print the log ouput of finished tasks.
/// Either print the logs of every task
/// or only print the logs of the specified tasks.
pub fn print_logs(tasks: BTreeMap<usize, Task>, cli_command: &SubCommand) {
    let (json, task_ids) = match cli_command {
        SubCommand::Log { json, task_ids } => (*json, task_ids.clone()),
        _ => panic!(
            "Got wrong Subcommand {:?} in print_log. This shouldn't happen",
            cli_command
        ),
    };
    if json {
        println!("{}", serde_json::to_string(&tasks).unwrap());
        return;
    }

    if task_ids.is_empty() && tasks.is_empty() {
        println!("There are no finished tasks");
        return;
    }

    if !task_ids.is_empty() && tasks.is_empty() {
        println!("There are no finished tasks for your specified ids");
        return;
    }

    let mut task_iter = tasks.iter().peekable();
    while let Some((_, task)) = task_iter.next() {
        print_log(task);
        if let Some((_, task)) = task_iter.peek() {
            if vec![TaskStatus::Done, TaskStatus::Failed, TaskStatus::Killed].contains(&task.status)
            {
                println!();
            }
        }
    }
}

/// Print the log of a single task.
pub fn print_log(task: &Task) {
    // We only show logs of finished tasks
    if !vec![TaskStatus::Done, TaskStatus::Failed, TaskStatus::Killed].contains(&task.status) {
        return;
    }

    let exit_status = match task.exit_code {
        Some(code) => match code {
            0 => style(format!("with exit code {}", code)).with(Color::Green),
            _ => style(format!("with exit code {}", code)).with(Color::Red),
        },
        None => style("failed to spawn".to_string()).with(Color::Red),
    };

    // Print task id and exit code
    print!(
        "{}",
        style(format!("Task {} ", task.id)).attribute(Attribute::Bold)
    );
    println!("{}", exit_status);

    // Print command and path
    println!("Command: {}", task.command);
    println!("Path: {}", task.path);

    if let Some(start) = task.start {
        println!("Start: {}", start.to_rfc2822());
    }
    if let Some(end) = task.end {
        println!("End: {}", end.to_rfc2822());
    }

    if let Some(stdout) = &task.stdout {
        if !stdout.is_empty() {
            println!(
                "{}",
                style("Std_out: ")
                    .with(Color::Green)
                    .attribute(Attribute::Bold)
            );
            println!("{}", stdout);
        }
    }

    if let Some(stderr) = &task.stderr {
        if !stderr.is_empty() {
            println!(
                "{}",
                style("Std_err: ")
                    .with(Color::Red)
                    .attribute(Attribute::Bold)
            );
            println!("{}", stderr);
        }
    }
}
