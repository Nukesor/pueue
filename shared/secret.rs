use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use anyhow::{bail, Result};
use rand::Rng;

/// Simple helper function to generate a random secret
pub fn read_shared_secret(path: &PathBuf) -> Result<Vec<u8>> {
    if !path.exists() {
        bail!("Couldn't find shared secret file. Did you start the daemon at least once?");
    }

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

/// Simple helper function to generate a random secret
pub fn init_shared_secret(path: &PathBuf) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    const PASSWORD_LEN: usize = 500;
    let mut rng = rand::thread_rng();

    let secret: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    let mut file = File::create(path)?;
    file.write_all(&secret.into_bytes())?;

    Ok(())
}
