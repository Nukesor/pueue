use std::fs::File;
use std::io::prelude::*;

use anyhow::Result;
use tempdir::TempDir;

use pueue_lib::settings::Settings;
use pueue_lib::state::State;

/// From 0.12.2 on, we aim to have full backward compatibility.
/// For this reason, an old v0.12.2 serialized state has been checked in.
///
/// We have to be able to restore from that state at all costs.
/// Everything else results in a breaking change and needs a major version change.
#[test]
fn test_restore_from_old_state() -> Result<()> {
    let old_state = include_str!("v0.12.2_state.json");

    let temp_dir = TempDir::new("pueue_lib")?;
    let temp_path = temp_dir.path();

    // Open v0.12.2 file and write old state to it.
    let temp_state_path = temp_dir.path().join("state.json");
    let mut file = File::create(&temp_state_path)?;
    file.write_all(old_state.as_bytes())?;

    let mut settings: Settings = Settings::default_config()?.try_into()?;
    settings.shared.pueue_directory = temp_path.to_path_buf();

    let mut state = State::new(&settings, None);
    if let Err(error) = state.restore() {
        println!("Failed to restore state in test: {:?}", error);
        assert!(false);
    }

    Ok(())
}
