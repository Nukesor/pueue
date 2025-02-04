use std::fs::File;
use std::io::{self, Stdout};

use crossterm::style::{Attribute, Color};

use pueue_lib::log::{get_log_file_handle, seek_to_last_lines};
use pueue_lib::settings::Settings;

use crate::client::display::OutputStyle;

/// The daemon didn't send any log output, thereby we didn't request any.
/// If that's the case, read the log file from the local pueue directory.
pub fn print_local_log(
    task_id: usize,
    style: &OutputStyle,
    settings: &Settings,
    lines: Option<usize>,
) {
    let mut file = match get_log_file_handle(task_id, &settings.shared.pueue_directory()) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to get log file handle: {err}");
            return;
        }
    };
    // Stdout handler to directly write log file output to io::stdout
    // without having to load anything into memory.
    let mut stdout = io::stdout();

    print_local_file(
        &mut stdout,
        &mut file,
        &lines,
        style.style_text("output:", Some(Color::Green), Some(Attribute::Bold)),
    );
}

/// Print a local log file of a task.
fn print_local_file(stdout: &mut Stdout, file: &mut File, lines: &Option<usize>, header: String) {
    if let Ok(metadata) = file.metadata() {
        if metadata.len() != 0 {
            // Indicates whether the full log output is shown or just the last part of it.
            let mut output_complete = true;

            // Only print the last lines if requested
            if let Some(lines) = lines {
                match seek_to_last_lines(file, *lines) {
                    Ok(complete) => output_complete = complete,
                    Err(err) => {
                        eprintln!("Failed reading local log file: {err}");
                        return;
                    }
                }
            }

            // Add a hint if we should limit the output to X lines **and** there are actually more
            // lines than that given limit.
            let mut line_info = String::new();
            if !output_complete {
                line_info = lines.map_or(String::new(), |lines| format!(" (last {lines} lines)"));
            }

            // Print a newline between the task information and the first output.
            eprintln!("\n{header}{line_info}");

            // Print everything
            if let Err(err) = io::copy(file, stdout) {
                eprintln!("Failed reading local log file: {err}");
            };
        }
    }
}
