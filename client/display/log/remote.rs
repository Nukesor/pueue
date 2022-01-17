use std::io;

use anyhow::Result;
use comfy_table::*;
use snap::read::FrameDecoder;

use pueue_lib::network::message::TaskLogMessage;

use crate::display::{colors::Colors, helper::*};

/// Prints log output received from the daemon.
/// We can safely call .unwrap() on output in here, since this
/// branch is always called after ensuring that it is `Some`.
pub fn print_remote_log(task_log: &TaskLogMessage, colors: &Colors) {
    if let Some(bytes) = task_log.output.as_ref() {
        if !bytes.is_empty() {
            let header = style_text("output: ", Some(colors.green()), Some(Attribute::Bold));
            println!("\n{header}",);

            if let Err(err) = decompress_and_print_remote_log(bytes) {
                println!("Error while parsing stdout: {err}");
            }
        }
    }
}

/// We cannot easily stream log output from the client to the daemon (yet).
/// Right now, the output is compressed in the daemon and sent as a single payload to the
/// client. In here, we take that payload, decompress it and stream it it directly to stdout.
fn decompress_and_print_remote_log(bytes: &[u8]) -> Result<()> {
    let mut decompressor = FrameDecoder::new(bytes);

    let stdout = io::stdout();
    let mut write = stdout.lock();
    io::copy(&mut decompressor, &mut write)?;

    Ok(())
}
