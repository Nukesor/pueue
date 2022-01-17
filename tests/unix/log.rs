use std::fs::read_to_string;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::{bail, Context, Result};
use pueue_lib::network::message::*;
use snap::read::FrameDecoder;
use tempfile::TempDir;

use crate::fixtures::*;
use crate::helper::*;

/// This function creates files `[1-20]` in the specified directory.
/// The return value is the expected output.
///
/// If `partial == true`, the expected output are only the last 5 lines.
fn create_test_files(path: &Path, partial: bool) -> Result<String> {
    // Convert numbers from 1 to 01, so they're correctly ordered when using `ls`.
    let names: Vec<String> = (0..20)
        .map(|number| {
            if number < 10 {
                let mut name = "0".to_string();
                name.push_str(&number.to_string());
                name
            } else {
                number.to_string()
            }
        })
        .collect();

    for name in &names {
        File::create(path.join(name.to_string()))?;
    }

    // Only return the last 5 lines if partial output is requested.
    if partial {
        return Ok((15..20).fold(String::new(), |mut full, name| {
            full.push_str(&name.to_string());
            full.push('\n');
            full
        }));
    }

    // Create the full expected output.
    let mut expected_output = names.join("\n");
    expected_output.push('\n');
    Ok(expected_output)
}

// Log output is send in a compressed form from the daemon.
// We have to unpack it first.
fn decompress_log(bytes: Vec<u8>) -> Result<String> {
    let mut decoder = FrameDecoder::new(&bytes[..]);
    let mut output = String::new();
    decoder
        .read_to_string(&mut output)
        .context("Failed to decompress remote log output")?;

    Ok(output)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Make sure that receiving partial output from the daemon works.
async fn test_full_log() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a temporary directory and put some files into it.
    let tempdir = TempDir::new().unwrap();
    let tempdir_path = tempdir.path();
    let expected_output =
        create_test_files(tempdir_path, false).context("Failed to create test files.")?;

    // Add a task that lists those files and wait for it to finish.
    let command = format!("ls {:?}", tempdir_path);
    assert_success(add_task(shared, &command, false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    // Request all log lines
    let log_message = LogRequestMessage {
        task_ids: vec![0],
        send_logs: true,
        lines: None,
    };
    let response = send_message(shared, Message::Log(log_message)).await?;
    let logs = match response {
        Message::LogResponse(logs) => logs,
        _ => bail!("Received non LogResponse: {:#?}", response),
    };

    // Get the received output
    let logs = logs.get(&0).unwrap();
    let output = logs
        .output
        .clone()
        .context("Didn't find output on TaskLogMessage")?;
    let output = decompress_log(output)?;

    // Make sure it's the same
    assert_eq!(output, expected_output);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Make sure that receiving partial output from the daemon works.
async fn test_partial_log() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Create a temporary directory and put some files into it.
    let tempdir = TempDir::new().unwrap();
    let tempdir_path = tempdir.path();
    let expected_output =
        create_test_files(tempdir_path, true).context("Failed to create test files.")?;

    // Add a task that lists those files and wait for it to finish.
    let command = format!("ls {:?}", tempdir_path);
    assert_success(add_task(shared, &command, false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    // Debug output to see what the file actually looks like:
    let real_log_path = shared.pueue_directory().join("task_logs").join("0.log");
    let content = read_to_string(real_log_path).context("Failed to read actual file")?;
    println!("Actual log file contents: \n{}", content);

    // Request a partial log for task 0
    let log_message = LogRequestMessage {
        task_ids: vec![0],
        send_logs: true,
        lines: Some(5),
    };
    let response = send_message(shared, Message::Log(log_message)).await?;
    let logs = match response {
        Message::LogResponse(logs) => logs,
        _ => bail!("Received non LogResponse: {:#?}", response),
    };

    // Get the received output
    let logs = logs.get(&0).unwrap();
    let output = logs
        .output
        .clone()
        .context("Didn't find output on TaskLogMessage")?;
    let output = decompress_log(output)?;

    // Make sure it's the same
    assert_eq!(output, expected_output);

    Ok(())
}
