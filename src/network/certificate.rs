use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Context, Result};
use log::info;
use rcgen::generate_simple_self_signed;

use crate::settings::Settings;

/// This the default certificates at the default `pueue_dir/certs` location.
pub fn create_certificates(settings: &Settings) -> Result<()> {
    let certs_dir = settings.shared.pueue_directory.join("certs");

    let daemon_cert_path = certs_dir.join("daemon.cert");
    let daemon_key_path = certs_dir.join("daemon.key");

    if daemon_key_path.exists() || daemon_cert_path.exists() {
        if !(daemon_key_path.exists() && daemon_cert_path.exists()) {
            bail!(
                "Not all default certificates exist, some are missing. \
                 Please fix your cert/key paths.\n \
                 You can also remove the `$pueue_directory/certs` directory \
                 and restart the daemon to create new certificates/keys."
            );
        }
        info!("All default keys do exist.");
        return Ok(());
    }

    let subject_alt_names = vec!["pueue.local".to_string(), "localhost".to_string()];

    let cert = generate_simple_self_signed(subject_alt_names).unwrap();
    // The certificate is now valid for localhost and the domain "hello.world.example"
    let ca_cert = cert
        .serialize_pem()
        .context("Failed to serialize daemon certificate.")?;
    write_file(ca_cert, "daemon cert", &daemon_cert_path)?;

    let ca_key = cert.serialize_private_key_pem();
    write_file(ca_key, "daemon key", &daemon_key_path)?;

    Ok(())
}

fn write_file(blob: String, name: &str, path: &Path) -> Result<()> {
    info!("Generate {}.", name);
    let error_message = format!("Cannot write default {}: {:?}", name, path);
    let mut file = File::create(path).context(error_message.clone())?;

    file.write_all(&blob.into_bytes()).context(error_message)?;

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
