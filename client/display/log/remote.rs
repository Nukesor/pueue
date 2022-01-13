use std::io;

use anyhow::Result;
use comfy_table::*;
use snap::read::FrameDecoder;

use pueue_lib::network::message::TaskLogMessage;

use crate::display::{colors::Colors, helper::*};

/// Prints log output received from the daemon.
/// We can safely call .unwrap() on stdout and stderr in here, since this
/// branch is always called after ensuring that both are `Some`.
pub fn print_remote_log(task_log: &TaskLogMessage, colors: &Colors) {
    // Save whether stdout was printed, so we can add a newline between outputs.
    if let Some(bytes) = task_log.stdout.as_ref() {
        if !bytes.is_empty() {
            let stdout_header = style_text("stdout: ", Some(colors.green()), Some(Attribute::Bold));
            println!("\n{stdout_header}",);

            if let Err(err) = decompress_and_print_remote_log(bytes) {
                println!("Error while parsing stdout: {err}");
            }
        }
    }

    if let Some(bytes) = task_log.stderr.as_ref() {
        if !bytes.is_empty() {
            let stderr_header = style_text("stderr: ", Some(colors.red()), Some(Attribute::Bold));
            println!("\n{stderr_header}");

            if let Err(err) = decompress_and_print_remote_log(bytes) {
                println!("Error while parsing stderr: {err}");
            };
        }
    }
}

/// We cannot easily stream log output from the client to the daemon (yet).
/// Right now, stdout and stderr are compressed in the daemon and sent as a single payload to the
/// client. In here, we take that payload, decompress it and stream it it directly to stdout.
fn decompress_and_print_remote_log(bytes: &[u8]) -> Result<()> {
    let mut decompressor = FrameDecoder::new(bytes);

    let stdout = io::stdout();
    let mut write = stdout.lock();
    io::copy(&mut decompressor, &mut write)?;

    Ok(())
}
