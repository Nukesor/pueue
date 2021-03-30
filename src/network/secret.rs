use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use anyhow::{bail, Context, Result};
use rand::{distributions::Alphanumeric, Rng};

/// Read the shared secret from a file.
pub fn read_shared_secret(path: &Path) -> Result<Vec<u8>> {
    if !path.exists() {
        bail!("Couldn't find shared secret file. Did you start the daemon at least once?");
    }

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

/// Generate a random secret and write it to a file.
pub fn init_shared_secret(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    const PASSWORD_LEN: usize = 512;
    let mut rng = rand::thread_rng();

    let secret: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(PASSWORD_LEN)
        .collect();

    let mut file = File::create(path)?;
    file.write_all(&secret.into_bytes())?;

    // Set proper file permissions for unix filesystems
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = file
            .metadata()
            .context("Failed to set secret file permissions")?
            .permissions();
        permissions.set_mode(0o640);
        std::fs::set_permissions(path, permissions)
            .context("Failed to set permissions on tls certificate")?;
    }

    Ok(())
}
