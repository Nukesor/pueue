use anyhow::{bail, Result};

use pueue_lib::network::message::*;
use pueue_lib::settings::*;
use pueue_lib::state::GroupStatus;

use super::*;

/// Waits for a status on a specific group.
pub async fn wait_for_group_status(
    shared: &Shared,
    group: &str,
    expected_status: GroupStatus,
) -> Result<()> {
    let state = get_state(shared).await?;

    // Give the daemon about 1 sec to shutdown.
    let tries = 20;
    let mut current_try = 0;

    while current_try < tries {
        // Process is still alive, wait a little longer
        if let Some(status) = state.groups.get(group) {
            if matches!(status, _expected_status) {
                return Ok(());
            }
        }

        sleep_ms(50);
        current_try += 1;
    }

    bail!("Group {group} didn't change to state {expected_status:?} after about 1 sec.",);
}

/// Create a new group with a specific amount of slots.
pub async fn add_group_with_slots(shared: &Shared, group_name: &str, slots: usize) -> Result<()> {
    let add_message = Message::Group(GroupMessage::Add {
        name: group_name.to_string(),
        parallel_tasks: Some(slots),
    });
    assert_success(send_message(shared, add_message.clone()).await?);
    wait_for_group(shared, group_name).await?;

    Ok(())
}
