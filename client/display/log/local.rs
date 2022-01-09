use std::fs::File;
use std::io::{self, Stdout};

use comfy_table::*;

use pueue_lib::log::{get_log_file_handles, seek_to_last_lines};
use pueue_lib::settings::Settings;

use crate::display::{colors::Colors, helper::*};

/// The daemon didn't send any log output, thereby we didn't request any.
/// If that's the case, read the log files from the local pueue directory
pub fn print_local_log(task_id: usize, colors: &Colors, settings: &Settings, lines: Option<usize>) {
    let (mut stdout_file, mut stderr_file) =
        match get_log_file_handles(task_id, &settings.shared.pueue_directory()) {
            Ok((stdout, stderr)) => (stdout, stderr),
            Err(err) => {
                println!("Failed to get log file handles: {}", err);
                return;
            }
        };
    // Stdout handler to directly write log file output to io::stdout
    // without having to load anything into memory.
    let mut stdout = io::stdout();

    print_local_file(
        &mut stdout,
        &mut stdout_file,
        &lines,
        style_text("stdout:", Some(colors.green()), Some(Attribute::Bold)),
    );

    print_local_file(
        &mut stdout,
        &mut stderr_file,
        &lines,
        style_text("stderr:", Some(colors.red()), Some(Attribute::Bold)),
    );
}

/// Print a local log file.
/// This is usually either the stdout or the stderr
fn print_local_file(stdout: &mut Stdout, file: &mut File, lines: &Option<usize>, text: String) {
    if let Ok(metadata) = file.metadata() {
        if metadata.len() != 0 {
            // Don't print a newline between the task information and the first output
            println!("\n{}", text);

            // Only print the last lines if requested
            if let Some(lines) = lines {
                if let Err(err) = seek_to_last_lines(file, *lines) {
                    println!("Failed reading local log file: {}", err);
                    return;
                }
            }

            // Print everything
            if let Err(err) = io::copy(file, stdout) {
                println!("Failed reading local log file: {}", err);
            };
        }
    }
}
