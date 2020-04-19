use ::comfy_table::presets::UTF8_HORIZONTAL_BORDERS_ONLY;
use ::comfy_table::*;
use ::crossterm::style::style;
use ::std::collections::BTreeMap;
use ::std::string::ToString;

use ::pueue::state::State;
use ::pueue::task::{Finished, GeneralState, Running, Startability, Task, TaskState};

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
    if json {
        println!("{}", serde_json::to_string(&state).unwrap());
        return;
    }

    let daemon_status = if state.running {
        style("Daemon status: running").with(Color::Green)
    } else {
        style("Daemon status: paused").with(Color::Yellow)
    };

    println!("{}", daemon_status);

    if state.tasks.len() == 0 {
        println!("\nTask list is empty. Add tasks with `pueue add -- [cmd]`");

        return;
    }

    let has_delayed_tasks = state
        .tasks
        .iter()
        .find(|(_id, task)| task.is_delayed())
        .is_some();

    let mut headers = vec![Cell::new("Index"), Cell::new("Status")];
    if has_delayed_tasks {
        headers.push(Cell::new("Enqueue At"));
    }
    headers.append(&mut vec![
        Cell::new("Exitcode"),
        Cell::new("Command"),
        Cell::new("Path"),
        Cell::new("Start"),
        Cell::new("End"),
    ]);

    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .load_preset(UTF8_HORIZONTAL_BORDERS_ONLY)
        .set_header(headers);

    for (id, task) in &state.tasks {
        let mut row = Row::new();
        // Add a row per time
        row.add_cell(Cell::new(&id.to_string()));

        // Add status cell and color depending on state
        let status_cell = Cell::new(match task.state() {
            TaskState::Waiting { .. } => match task.start_info(&state) {
                Startability::Ready => "Ready to start".to_string(),
                Startability::Dependencies(dep) => format!(
                    "Waiting for tasks {}",
                    dep.iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Startability::DependenciesFailure(failed) => {
                    format!("Dependencies {} failed", failed)
                }
                Startability::Waiting(_) => "Waiting".to_string(),
                Startability::Locked => "Locked".to_string(),
                Startability::Stashed => "Stashed".to_string(),
                Startability::Unknown => panic!("Should have a known startability {:?}", task.state()),
            },
            TaskState::Running {
                status: Running::Running,
                ..
            } => "Running".to_string(),
            TaskState::Running {
                status: Running::Paused,
                ..
            } => "Paused".to_string(),
            TaskState::Finished {
                status: Finished::Done,
                ..
            } => "Done".to_string(),
            TaskState::Finished {
                status: Finished::Failed(_),
                ..
            } => "Failed".to_string(),
            TaskState::Finished {
                status: Finished::Killed,
                ..
            } => "Killed".to_string(),
            TaskState::Finished {
                status: Finished::Errored,
                ..
            } => "Internal Error".to_string(),
            TaskState::Finished {
                status: Finished::UnableToSpawn(_),
                ..
            } => "Unable to spawn".to_string(),
        });

        let status_cell = match task.general_state(&state) {
            GeneralState::Healthy => status_cell.fg(Color::Green),
            GeneralState::Failed => status_cell.fg(Color::Red),
            GeneralState::Paused => status_cell.fg(Color::White),
            GeneralState::Waiting => status_cell.fg(Color::Yellow),
        };
        row.add_cell(status_cell);

        if has_delayed_tasks {
            if let TaskState::Waiting {
                enqueue_at: Some(enqueue_at),
                ..
            } = task.state()
            {
                row.add_cell(Cell::new(enqueue_at.format("%Y-%m-%d\n%H:%M:%S")));
            } else {
                row.add_cell(Cell::new(""));
            }
        }

        // Match the color of the exit code
        // If the exit_code is none, it has been killed by the task handler.
        row.add_cell(match task.state() {
            TaskState::Finished {
                status: Finished::Done,
                ..
            } => Cell::new("0").fg(Color::Green),
            TaskState::Finished {
                status: Finished::Failed(exit_code),
                ..
            } => Cell::new(&exit_code.to_string()).fg(Color::Red),
            TaskState::Finished {
                status: Finished::Killed,
                ..
            } => Cell::new("Killed").fg(Color::Yellow),
            _ => Cell::new(""),
        });

        // Add command and path
        row.add_cell(Cell::new(&task.command));
        row.add_cell(Cell::new(&task.path));

        // Add start time, if already set
        row.add_cell(Cell::new(match task.state() {
            TaskState::Running { start, .. } => start.format("%H:%M").to_string(),
            TaskState::Finished { start, .. } => start.format("%H:%M").to_string(),
            _ => String::new(),
        }));

        // Add finish time, if already set
        row.add_cell(Cell::new(match task.state() {
            TaskState::Running { .. } => "...".to_string(),
            TaskState::Finished { end, .. } => end.format("%H:%M").to_string(),
            _ => String::new(),
        }));

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
            if task.is_finished() {
                println!("");
            }
        }
    }
}

/// Print the log of a single task.
pub fn print_log(task: &Task) {
    match task.state() {
        TaskState::Finished {
            start,
            end,
            status,
            stdout,
            stderr,
        } => {
            println!(
                "{}",
                match status {
                    Finished::UnableToSpawn(err) => style(format!("Failed to spawn {}", err)).with(Color::Red),
                    Finished::Errored => style("Unknown internal error".to_string()).with(Color::Red),
                    Finished::Killed => style("Killed".to_string()).with(Color::Red),
                    Finished::Done =>
                        style(format!("with exit code 0")).with(Color::Green),
                    Finished::Failed(exit_code) =>
                        style(format!("with exit code {}", exit_code)).with(Color::Red),
                }
            );

            println!("Start: {}", start.to_rfc2822());
            println!("End: {}", end.to_rfc2822());

            if !stdout.is_empty() {
                println!(
                    "{}",
                    style("Std_out: ")
                        .with(Color::Green)
                        .attribute(Attribute::Bold)
                );
                println!("{}", stdout);
            };

            if !stderr.is_empty() {
                println!(
                    "{}",
                    style("Std_err: ")
                        .with(Color::Red)
                        .attribute(Attribute::Bold)
                );
                println!("{}", stderr);
            };
        }

        // We only show logs of finished tasks
        _ => (),
    };
}
