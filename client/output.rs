use ::std::string::ToString;
use ::crossterm::style::{style, Color, Attribute};
use ::comfy_table::prelude::*;
use ::comfy_table::style::presets::UTF8_FULL;

use ::pueue::message::*;
use ::pueue::state::State;
use ::pueue::task::TaskStatus;

pub fn print_success(message: String) {
    println!("{}", message);
}

pub fn print_error(message: String) {
    let styled = style(message).with(Color::Red);
    println!("{}", styled);
}

/// Print the current state of the daemon in a nicely formatted table
pub fn print_state(message: Message, json: bool) {
    let state = match message {
        Message::StatusResponse(state) => state,
        _ => return,
    };
    if json {
        println!("{}", serde_json::to_string(&state).unwrap());
        return;
    }

    if state.tasks.len() == 0 {
        println!("Task list is empty. Add tasks with `pueue add -- [cmd]`");

        return;
    }
    let mut daemon_status = if state.running {
        style("Daemon status: running")
    } else {
        style("Daemon status: ")
    };

    if state.running {
        daemon_status = daemon_status.with(Color::Green);
    } else {
        daemon_status = daemon_status.with(Color::Yellow);
    }
    println!("{}", daemon_status);

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic)
        .load_preset(UTF8_FULL)
        .set_header(vec![
        Cell::new("Index"),
        Cell::new("Status"),
        Cell::new("Exitcode"),
        Cell::new("Command"),
        Cell::new("Path"),
        Cell::new("Start"),
        Cell::new("End"),
    ]);

    for (id, task) in state.tasks {
        let mut row = Row::new();
        // Add a row per time
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

        // Match the color of the exit code
        // If the exit_code is none, it has been killed by the task handler.
        match task.exit_code {
            Some(code) => {
                // Everything that's not 0, is failed task
                if code == 0 {
                    row.add_cell(
                        Cell::new(&code.to_string()).fg(Color::Green),
                    );
                } else {
                    row.add_cell(
                        Cell::new(&code.to_string()).fg(Color::Red)
                    );
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
pub fn print_logs(message: Message, task_ids: Vec<usize>, json: bool) {
    let state = match message {
        Message::StatusResponse(state) => state,
        _ => return,
    };
    if json {
        println!("{}", serde_json::to_string(&state).unwrap());
        return;
    }

    if !task_ids.is_empty() {
        for task_id in task_ids {
            print_log(task_id, &state);
        }
    } else {
        for task_id in state.tasks.keys() {
            print_log(*task_id, &state);
        }
    }
}

/// Print the log of a single task.
pub fn print_log(task_id: usize, state: &State) {
    let task = match state.tasks.get(&task_id) {
        Some(task) => task,
        None => return,
    };

    // We only show logs of finished tasks
    if !vec![TaskStatus::Done, TaskStatus::Failed].contains(&task.status) {
        return;
    }

    let exit_status = match task.exit_code {
        Some(code) => match code {
            0 => style(format!("with exit code {}", code)).with(Color::Green),
            _ => style(format!("with exit code {}", code)).with(Color::Red),
        },
        None => style("failed to Spawn".to_string()).with(Color::Red),
    };

    // Print task id and exit code
    println!("\n");
    print!("{}", style(format!("Task {} ", task.id)).attribute(Attribute::Bold));
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
            println!("{}", style("Std_out: ").with(Color::Green).attribute(Attribute::Bold));
            println!("{}", stdout);
        }
    }

    if let Some(stderr) = &task.stderr {
        if !stderr.is_empty() {
            println!("{}", style("Std_err: ").with(Color::Red).attribute(Attribute::Bold));
            println!("{}", stderr);
        }
    }
}
