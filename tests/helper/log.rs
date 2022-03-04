use std::io::Read;

use anyhow::{bail, Context, Result};
use pueue_lib::network::message::*;
use pueue_lib::settings::*;
use snap::read::FrameDecoder;

use super::*;

// Log output is send in a compressed form from the daemon.
// We have to unpack it first.
pub fn decompress_log(bytes: Vec<u8>) -> Result<String> {
    let mut decoder = FrameDecoder::new(&bytes[..]);
    let mut output = String::new();
    decoder
        .read_to_string(&mut output)
        .context("Failed to decompress remote log output")?;

    Ok(output)
}

/// Convenience function to get the log of a specific task.
/// `lines: None` requests all log lines.
pub async fn get_task_log(shared: &Shared, task_id: usize, lines: Option<usize>) -> Result<String> {
    let message = Message::Log(LogRequestMessage {
        task_ids: vec![task_id],
        send_logs: true,
        lines,
    });
    let response = send_message(shared, message).await?;

    let mut logs = match response {
        Message::LogResponse(logs) => logs,
        _ => bail!("Didn't get log response response in get_state"),
    };

    let log = logs
        .remove(&task_id)
        .context("Didn't find log of requested task")?;
    let bytes = log
        .output
        .context("Didn't get log output even though requested.")?;

    decompress_log(bytes)
}
