#![allow(dead_code)]
use anyhow::Result;
use tokio::io::{self, AsyncWriteExt};

pub mod daemon;
pub mod fixtures;
pub mod network;
pub mod wait;

pub use daemon::*;
pub use network::*;
pub use wait::*;

pub fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

pub async fn async_println(out: String) -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .write_all(out.as_bytes())
        .await
        .expect("Failed to write to stdout.");
    stdout.flush().await?;

    Ok(())
}
