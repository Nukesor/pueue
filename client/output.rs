use std::string::ToString;
use prettytable::{Table, Row, Cell, Attr, color, format};

use ::pueue::state::State;
use ::pueue::task::TaskStatus;

/// Print the current state of the daemon in a nicely formatted table
pub fn print_state(state: State) {

    if state.tasks.len()  == 0 {
        println!("Task list is empty. Add tasks with `pueue add -- [cmd]`");

        return
    }

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
                    row.add_cell(Cell::new(&code.to_string())
                        .with_style(Attr::ForegroundColor(color::GREEN)));
                } else {
                    row.add_cell(Cell::new(&code.to_string())
                        .with_style(Attr::ForegroundColor(color::RED)));
                }
            },
            None => {
                if task.is_done() {
                    row.add_cell(Cell::new("Killed")
                        .with_style(Attr::ForegroundColor(color::RED)));
                } else {
                    row.add_cell(Cell::new(""));
                }
            },
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
