//! This module contains helper functions, which are used by both, the client and daemon tests.
use std::process::Output;

pub use pueue_lib::state::PUEUE_DEFAULT_GROUP;
use tokio::io::{self, AsyncWriteExt};

use crate::internal_prelude::*;

mod asserts;
mod daemon;
mod factories;
mod fixtures;
mod lockfile;
mod log;
mod network;
mod state;
mod task;
mod wait;

pub use asserts::*;
pub use daemon::*;
pub use factories::*;
pub use fixtures::*;
pub use lockfile::*;
pub use network::*;
pub use state::*;
pub use task::*;
pub use wait::*;

pub use self::log::*;

// Global acceptable test timeout
const TIMEOUT: u64 = 5000;

/// Use this function to enable log output for in-runtime daemon output.
#[allow(dead_code)]
pub fn enable_logger() {
    pueue::tracing::install_tracing(3)
        .expect("Couldn't init tracing for test, have you initialised tracing twice?");
}

/// A helper function to sleep for ms time.
/// Only used to avoid the biolerplate of importing the same stuff all over the place.
pub async fn sleep_ms(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

/// A small helper function, which instantly writes the given string to stdout with a newline.
/// Useful for debugging async tests.
#[allow(dead_code)]
pub async fn async_println(out: &str) -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .write_all(out.as_bytes())
        .await
        .expect("Failed to write to stdout.");

    stdout
        .write_all("\n".as_bytes())
        .await
        .expect("Failed to write to stdout.");
    stdout.flush().await?;

    Ok(())
}

/// Take some process output and simply print it.
#[allow(dead_code)]
pub fn print_output(output: &Output) -> Result<()> {
    let stdout = output.stdout.clone();
    let stderr = output.stderr.clone();
    let out = String::from_utf8(stdout).context("Got invalid utf8 as stdout!")?;
    let err = String::from_utf8(stderr).context("Got invalid utf8 as stderr!")?;

    println!("Stdout:\n{out}");
    println!("\nStderr:\n{err}");

    Ok(())
}

pub trait CommandOutcome {
    fn success(self) -> Result<Output>;

    fn failure(self) -> Result<Output>;
}

impl CommandOutcome for Output {
    fn success(self) -> Result<Output> {
        if !self.status.success() {
            print_output(&self)?;
            bail!("Command failed, see log output.")
        }

        Ok(self)
    }

    fn failure(self) -> Result<Output> {
        if self.status.success() {
            print_output(&self)?;
            bail!("Command succeeded, even though it should've failed. See log output.")
        }

        Ok(self)
    }
}
