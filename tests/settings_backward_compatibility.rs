use std::path::PathBuf;

use anyhow::{Context, Result};

use pueue_lib::settings::Settings;

/// From 0.15.0 on, we aim to have full backward compatibility.
/// For this reason, an old (slightly modified) v0.15.0 serialized settings file
/// has been checked in.
///
/// We have to be able to restore from that config at all costs.
/// Everything else results in a breaking change and needs a major version change.
///
/// On top of simply having an old settings, I also removed a few default fields.
/// This should be handled as well.
#[test]
fn test_restore_from_old_state() -> Result<()> {
    better_panic::install();
    let old_settings_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("v0.15.0_settings.yml");

    // Open v0.15.0 file and ensure the settings file can be read.
    let settings: Settings = Settings::read_with_defaults(true, &Some(old_settings_path))
        .context("Failed to read old config with defaults:")?;

    assert!(settings.daemon.groups.get("default").is_some());

    Ok(())
}
