use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use rand::{distributions::Alphanumeric, Rng};

use crate::error::Error;

/// Read the shared secret from a file.
pub fn read_shared_secret(path: &Path) -> Result<Vec<u8>, Error> {
    if !path.exists() {
        return Err(Error::FileNotFound(
            "Secret. Did you start the daemon at least once?".into(),
        ));
    }

    let mut file = File::open(path)
        .map_err(|err| Error::IoPathError(path.to_path_buf(), "opening secret file", err))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|err| Error::IoPathError(path.to_path_buf(), "reading secret file", err))?;

    Ok(buffer)
}

/// Generate a random secret and write it to a file.
pub fn init_shared_secret(path: &Path) -> Result<(), Error> {
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

    let mut file = File::create(&path)
        .map_err(|err| Error::IoPathError(path.to_path_buf(), "creating shared secret", err))?;
    file.write_all(&secret.into_bytes())
        .map_err(|err| Error::IoPathError(path.to_path_buf(), "writing shared secret", err))?;

    // Set proper file permissions for unix filesystems
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = file
            .metadata()
            .map_err(|err| {
                Error::IoPathError(path.to_path_buf(), "reading secret file metadata", err)
            })?
            .permissions();
        permissions.set_mode(0o640);
        std::fs::set_permissions(path, permissions).map_err(|err| {
            Error::IoPathError(path.to_path_buf(), "setting secret file permissions", err)
        })?;
    }

    Ok(())
}
