use std::{fs, path::PathBuf};

use anyhow::{Context, Result};

use pueue_lib::state::{GroupStatus, State, PUEUE_DEFAULT_GROUP};

/// From 0.18.0 on, we aim to have full backward compatibility for our state deserialization.
/// For this reason, an old (slightly modified) v0.18.0 serialized state has been checked in.
///
/// **Warning**: This is only one part of our state tests.
///              There is another full test suite in the `pueue` project, which deals with domain
///              specific state restoration logic. This test only checks, whether we can
///              deserialize old state files.
///
/// We have to be able to restore from that state at all costs.
/// Everything else results in a breaking change and needs a major version change.
/// (For `pueue_lib` as well as `pueue`!
///
/// On top of simply having an old state, I also removed a few default fields.
/// This should be handled as well.
#[test]
fn test_restore_from_old_state() -> Result<()> {
    better_panic::install();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("v0.18.0_state.json");

    // Try to load the file.
    let data = fs::read_to_string(&path).context("State restore: Failed to read file:\n\n{}")?;
    // Try to deserialize the state file.
    let state: State = serde_json::from_str(&data).context("Failed to deserialize state.")?;

    // Make sure the groups are loaded.
    assert!(
        state.groups.get(PUEUE_DEFAULT_GROUP).is_some(),
        "Group 'default' should exist."
    );
    assert_eq!(
        state.groups.get(PUEUE_DEFAULT_GROUP).unwrap().status,
        GroupStatus::Paused
    );
    assert!(
        state.groups.get("test").is_some(),
        "Group 'test' should exist"
    );
    assert_eq!(
        state.groups.get("test").unwrap().status,
        GroupStatus::Paused
    );

    assert!(state.tasks.get(&3).is_some(), "Task 3 should exist");
    assert_eq!(state.tasks.get(&3).unwrap().command, "ls stash_it");

    Ok(())
}
