use ::prettytable::{color, format, Attr, Cell, Row, Table};
use ::std::string::ToString;
use ::termion::{color as t_color, style};

use ::pueue::message::*;
use ::pueue::state::State;
use ::pueue::task::TaskStatus;

pub fn print_success(message: String) {
    println!("{}", message);
}

pub fn print_error(message: String) {
    println!("{}{}", t_color::Fg(t_color::Red), message);
}

/// Print the current state of the daemon in a nicely formatted table
pub fn print_state(message: Message, json: bool) {
    let state = match message {
        Message::StatusResponse(state) => state,
        _ => return,
    };
    if json {
        println!("{}", serde_json::to_string(&state).unwrap());
        return
    }

    if state.tasks.len() == 0 {
        println!("Task list is empty. Add tasks with `pueue add -- [cmd]`");

        return;
    }
    let mut daemon_status = String::from("Daemon status: ");
    if state.running {
        daemon_status.push_str(&format!("{}", t_color::Fg(t_color::Green)));
        daemon_status.push_str("running");
    } else {
        daemon_status.push_str(&format!("{}", t_color::Fg(t_color::Yellow)));
        daemon_status.push_str("paused");
    }
    daemon_status.push_str(&format!("{}", style::Reset));

    println!("{}", daemon_status);

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_COLSEP);
    let header_row = Row::new(vec![
        Cell::new("Index"),
        Cell::new("Status"),
        Cell::new("Exitcode"),
        Cell::new("Command"),
        Cell::new("Path"),
        Cell::new("Start"),
        Cell::new("End"),
    ]);
    table.set_titles(header_row);

    for (id, task) in state.tasks {
        let mut row = Row::new(vec![]);
        // Add a row per time
        row.add_cell(Cell::new(&id.to_string()));

        // Add status cell and color depending on state
        let status_cell = Cell::new(&task.status.to_string());
        let status_style = match task.status {
            TaskStatus::Running | TaskStatus::Done => Attr::ForegroundColor(color::GREEN),
            TaskStatus::Failed => Attr::ForegroundColor(color::RED),
            _ => Attr::ForegroundColor(color::YELLOW),
        };
        row.add_cell(status_cell.with_style(status_style));

        // Match the color of the exit code
        // If the exit_code is none, it has been killed by the task handler.
        match task.exit_code {
            Some(code) => {
                // Everything that's not 0, is failed task
                if code == 0 {
                    row.add_cell(
                        Cell::new(&code.to_string())
                            .with_style(Attr::ForegroundColor(color::GREEN)),
                    );
                } else {
                    row.add_cell(
                        Cell::new(&code.to_string()).with_style(Attr::ForegroundColor(color::RED)),
                    );
                }
            }
            None => {
                if task.is_done() {
                    row.add_cell(Cell::new("Killed").with_style(Attr::ForegroundColor(color::RED)));
                } else {
                    row.add_cell(Cell::new(""));
                }
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
    table.printstd();
}

/// Print the log ouput of finished tasks.
/// Either print the logs of every task
/// or only print the logs of the specified tasks.
pub fn print_logs(message: Message, task_ids: Option<Vec<i32>>, json: bool) {
    let state = match message {
        Message::StatusResponse(state) => state,
        _ => return,
    };
    if json {
        println!("{}", serde_json::to_string(&state).unwrap());
        return
    }

    match task_ids {
        Some(task_ids) => {
            for task_id in task_ids {
                print_log(task_id, &state);
            }
        }
        None => {
            for task_id in state.tasks.keys() {
                print_log(*task_id, &state);
            }
        }
    }
}

/// Print the log of a single task.
pub fn print_log(task_id: i32, state: &State) {
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
            0 => format!(
                "with exit code {}{}{}",
                t_color::Fg(t_color::Green),
                code,
                style::Reset
            ),
            _ => format!(
                "with exit code {}{}{}",
                t_color::Fg(t_color::Red),
                code,
                style::Reset
            ),
        },
        None => format!(
            "{}{}{}",
            t_color::Fg(t_color::Red),
            "failed to Spawn",
            style::Reset
        ),
    };

    println!("\n");
    println!(
        "{}Task {} {}{}",
        style::Bold,
        task.id,
        exit_status,
        style::Reset
    );
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
                "\n{}{}Std_out:{}",
                style::Bold,
                t_color::Fg(t_color::Green),
                style::Reset
            );
            println!("{}", stdout);
        }
    }

    if let Some(stderr) = &task.stderr {
        if !stderr.is_empty() {
            println!(
                "\n{}{}Std_err:{}",
                style::Bold,
                t_color::Fg(t_color::Red),
                style::Reset
            );
            println!("{}", stderr);
        }
    }
}
