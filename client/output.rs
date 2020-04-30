use ::anyhow::Result;
use ::base64::read::DecoderReader;
use ::brotli::BrotliDecompress;
use ::comfy_table::presets::UTF8_HORIZONTAL_BORDERS_ONLY;
use ::comfy_table::*;
use ::crossterm::style::style;
use ::std::collections::BTreeMap;
use ::std::io;
use ::std::string::ToString;

use ::pueue::message::TaskLogMessage;
use ::pueue::state::State;
use ::pueue::task::{TaskResult, TaskStatus};

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

        // Determine the human readable task status representation and the respective color
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

        // Match the color of the exit code
        // If the exit_code is none, it has been killed by the task handler.
        let exit_code_cell = match task.result {
            Some(TaskResult::Success) => Cell::new("0").fg(Color::Green),
            Some(TaskResult::Failed(code)) => Cell::new(&code.to_string()).fg(Color::Red),
            _ => Cell::new(""),
        };
        row.add_cell(exit_code_cell);

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
pub fn print_logs(mut task_logs: BTreeMap<usize, TaskLogMessage>, cli_command: &SubCommand) {
    let (json, task_ids) = match cli_command {
        SubCommand::Log { json, task_ids } => (*json, task_ids.clone()),
        _ => panic!(
            "Got wrong Subcommand {:?} in print_log. This shouldn't happen",
            cli_command
        ),
    };
    if json {
        println!("{}", serde_json::to_string(&task_logs).unwrap());
        return;
    }

    if task_ids.is_empty() && task_logs.is_empty() {
        println!("There are no finished task_logs");
        return;
    }

    if !task_ids.is_empty() && task_logs.is_empty() {
        println!("There are no finished task_logs for your specified ids");
        return;
    }

    let mut task_iter = task_logs.iter_mut().peekable();
    while let Some((_, mut task_log)) = task_iter.next() {
        print_log(&mut task_log);

        // Add a newline if there is another task that's going to be printed
        if let Some((_, task_log)) = task_iter.peek() {
            if task_log.task.status == TaskStatus::Done {
                println!();
            }
        }
    }
}

/// Print the log of a single task.
pub fn print_log(task_log: &mut TaskLogMessage) {
    let task = &task_log.task;
    // We only show logs of finished tasks
    if task.status != TaskStatus::Done {
        return;
    }

    // Print task id and exit code
    let task_text = style(format!("Task {} ", task.id)).attribute(Attribute::Bold);
    let exit_status = match &task.result {
        Some(TaskResult::Success) => style(format!("with exit code 0")).with(Color::Green),
        Some(TaskResult::Failed(exit_code)) => {
            style(format!("with exit code {}", exit_code)).with(Color::Red)
        }
        Some(TaskResult::FailedToSpawn(err)) => {
            style(format!("failed to spawn: {}", err)).with(Color::Red)
        }
        Some(TaskResult::Killed) => style("killed by system or user".to_string()).with(Color::Red),
        Some(TaskResult::DependencyFailed) => {
            style("dependency failed".to_string()).with(Color::Red)
        }
        None => panic!("Got a 'Done' task without a task result. Please report this bug."),
    };
    print!("{} {}", task_text, exit_status);

    // Print command and path
    println!("Command: {}", task.command);
    println!("Path: {}", task.path);

    if let Some(start) = task.start {
        println!("Start: {}", start.to_rfc2822());
    }
    if let Some(end) = task.end {
        println!("End: {}", end.to_rfc2822());
    }

    if !task_log.stdout.is_empty() {
        if let Err(err) = print_task_stdout(task_log) {
            println!("Error while parsing stdout: {}", err);
        }
    }

    if !task_log.stderr.is_empty() {
        if let Err(err) = print_task_stderr(task_log) {
            println!("Error while parsing stderr: {}", err);
        };
    }
}

/// Pritn the stdout of a finished process
/// The logs are compressed using Brotli and then encoded to Base64
pub fn print_task_stdout(task_log: &mut TaskLogMessage) -> Result<()> {
    let mut bytes = task_log.stdout.as_bytes();
    // Minimum empty base64 encoded message length
    if bytes.len() <= 4 {
        return Ok(());
    }

    println!(
        "{}",
        style("Std_out: ")
            .with(Color::Green)
            .attribute(Attribute::Bold)
    );
    let mut stderr_base64_decoder = DecoderReader::new(&mut bytes, base64::STANDARD);
    BrotliDecompress(&mut stderr_base64_decoder, &mut io::stdout())?;

    Ok(())
}

/// Print the stderr of a finished process
/// The logs are compressed using Brotli and then encoded to Base64
pub fn print_task_stderr(task_log: &mut TaskLogMessage) -> Result<()> {
    let mut bytes = task_log.stderr.as_bytes();
    // Minimum empty base64 encoded message length
    if bytes.len() <= 4 {
        return Ok(());
    }

    println!(
        "{}",
        style("Std_err: ")
            .with(Color::Red)
            .attribute(Attribute::Bold)
    );
    let mut stderr_base64_decoder = DecoderReader::new(&mut bytes, base64::STANDARD);
    BrotliDecompress(&mut stderr_base64_decoder, &mut io::stdout())?;

    Ok(())
}
