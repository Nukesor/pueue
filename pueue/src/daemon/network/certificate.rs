use std::{fs::OpenOptions, io::Write, path::Path};

use rcgen::{CertifiedKey, generate_simple_self_signed};

use crate::internal_prelude::*;
use pueue_lib::{error::Error, settings::Shared};

/// This the default certificates at the default `pueue_dir/certs` location.
pub fn create_certificates(shared_settings: &Shared) -> Result<(), Error> {
    let daemon_cert_path = shared_settings.daemon_cert();
    let daemon_key_path = shared_settings.daemon_key();

    if daemon_key_path.exists() || daemon_cert_path.exists() {
        if !(daemon_key_path.exists() && daemon_cert_path.exists()) {
            return Err(Error::CertificateFailure(
                "Not all default certificates exist, some are missing. \
                 Please fix your cert/key paths.\n \
                 You can also remove the `$pueue_directory/certs` directory \
                 and restart the daemon to create new certificates/keys."
                    .into(),
            ));
        }
        info!("All default keys do exist.");
        return Ok(());
    }

    let subject_alt_names = vec!["pueue.local".to_string(), "localhost".to_string()];

    let CertifiedKey { cert, signing_key } = generate_simple_self_signed(subject_alt_names)
        .map_err(|_| {
            Error::CertificateFailure("Failed to generate self-signed daemon certificate.".into())
        })?;
    // The certificate is now valid for localhost
    let ca_cert = cert.pem();
    write_file(ca_cert, "daemon cert", &daemon_cert_path, 0o640)?;

    // The key is only ever read by the daemon itself, so it doesn't need to be group readable.
    let ca_key = signing_key.serialize_pem();
    write_file(ca_key, "daemon key", &daemon_key_path, 0o600)?;

    Ok(())
}

/// Write a certificate or key with a specific mode.
#[cfg_attr(target_os = "windows", allow(unused_variables))]
fn write_file(blob: String, name: &str, path: &Path, mode: u32) -> Result<(), Error> {
    info!("Generate {name}.");

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }

    let mut file = options
        .open(path)
        .map_err(|err| Error::IoPathError(path.to_path_buf(), "creating certificate", err))?;

    file.write_all(&blob.into_bytes())
        .map_err(|err| Error::IoPathError(path.to_path_buf(), "writing certificate", err))?;

    Ok(())
}
