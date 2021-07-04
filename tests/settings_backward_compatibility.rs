use std::path::PathBuf;

use anyhow::{Context, Result};

use pueue_lib::settings::Settings;

/// From 0.12.2 on, we aim to have full backward compatibility.
/// For this reason, an old v0.12.2 serialized state has been checked in.
///
/// We have to be able to restore from that state at all costs.
/// Everything else results in a breaking change and needs a major version change.
///
/// On top of simply having an old state, I also added a few non-existing fields.
/// This should be handled as well.
#[test]
fn test_restore_from_old_state() -> Result<()> {
    better_panic::install();
    let old_settings_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("v0.15.0_settings.yml");

    // Open v0.12.2 file and write old state to it.
    let _: Settings = Settings::read_with_defaults(true, &Some(old_settings_path))
        .context("Failed to read old config with defaults:")?;

    Ok(())
}
