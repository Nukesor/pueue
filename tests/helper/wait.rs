use anyhow::{bail, Result};

use pueue_lib::settings::Shared;
use pueue_lib::task::{Task, TaskStatus};

use super::{get_state, sleep_ms};

/// This is a small helper function, which checks in very short intervals, whether a task showed up
/// in the daemon or not. This is necessary to prevent always long or potentially flaky timeouts in
/// our tests.
///
/// Using continuous lookups, we can allow long waiting times, while still having fast tests if
/// things don't take that long.
pub async fn wait_for_task(shared: &Shared, task_id: usize) -> Result<()> {
    let tries = 20;
    let mut current_try = 0;
    while current_try <= tries {
        let state = get_state(shared).await?;
        if !state.tasks.contains_key(&task_id) {
            current_try += 1;
            sleep_ms(50);
            continue;
        }

        return Ok(());
    }

    bail!("Task {} didn't show up in about 1 second.", task_id)
}

/// This is a small helper function, which checks in very short intervals, whether a task changed
/// it's state or not. This is necessary to prevent always long or potentially flaky timeouts in
/// our tests.
///
/// Using continuous lookups, we can allow long waiting times, while still having fast tests if
/// things don't take that long.
pub async fn wait_for_status_change(
    shared: &Shared,
    task_id: usize,
    original_status: TaskStatus,
) -> Result<()> {
    let tries = 20;
    let mut current_try = 0;
    while current_try <= tries {
        let state = get_state(shared).await?;
        match state.tasks.get(&task_id) {
            Some(task) => {
                // The status changed. We can give our ok!
                if task.status != original_status {
                    return Ok(());
                }

                // The status didn't change. Try again.
                current_try += 1;
                sleep_ms(50);
                continue;
            }
            None => {
                bail!(
                    "Couldn't find task {} while waiting for status change",
                    task_id
                )
            }
        }
    }

    bail!("Task {} didn't change state in about 1 second.", task_id)
}

/// This is a small helper function, which checks in very short intervals, whether a task fulfills
/// a certain criteria. This is necessary to prevent long or potentially flaky timeouts in our tests.
///
/// Using continuous lookups, we can allow long waiting times, while still having fast tests if
/// things don't take that long.
/// This is used in integration tests to wait for state changes, i.e. when killing a task.
pub async fn wait_for_task_condition<F>(shared: &Shared, task_id: usize, condition: F) -> Result<()>
where
    F: Fn(&Task) -> bool,
{
    let tries = 20;
    let mut current_try = 0;
    while current_try <= tries {
        let state = get_state(shared).await?;
        match state.tasks.get(&task_id) {
            Some(task) => {
                // Check if the condition is met.
                // If it isn't, continue
                if condition(task) {
                    return Ok(());
                }

                // The status didn't change to target. Try again.
                current_try += 1;
                sleep_ms(50);
                continue;
            }
            None => {
                bail!("Couldn't find task {} while waiting for condition", task_id)
            }
        }
    }
    bail!(
        "Task {} didn't fulfill condition after about 1 second.",
        task_id,
    )
}

/// This is a small helper function, which checks in very short intervals, whether a group has been
/// initialized or not. This is necessary, as group creation became an asynchronous task.
///
/// Using continuous lookups, we can allow long waiting times, while still having fast tests if
/// things don't take that long.
pub async fn wait_for_group(shared: &Shared, group: &str) -> Result<()> {
    let tries = 20;
    let mut current_try = 0;
    while current_try <= tries {
        let state = get_state(shared).await?;
        if !state.groups.contains_key(group) {
            current_try += 1;
            sleep_ms(50);
            continue;
        }

        return Ok(());
    }

    bail!("Group {} didn't show up in about 1 second.", group)
}
