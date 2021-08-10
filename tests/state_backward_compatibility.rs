use std::fs::File;
use std::io::prelude::*;

use anyhow::{Context, Result};
use pueue_daemon_lib::state_helper::restore_state;
use tempfile::TempDir;

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
    let old_state = include_str!("data/v1.0.0_state.json");

    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Open v0.12.2 file and write old state to it.
    let temp_state_path = temp_dir.path().join("state.json");
    let mut file = File::create(&temp_state_path)?;
    file.write_all(old_state.as_bytes())?;

    let mut settings: Settings = Settings::default_config()?.try_into()?;
    settings.shared.pueue_directory = temp_path.to_path_buf();

    let state = restore_state(&settings.shared.pueue_directory())
        .context("Failed to restore state in test")?;

    assert!(state.is_some());

    Ok(())
}
