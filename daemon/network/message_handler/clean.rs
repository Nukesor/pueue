use pueue::log::clean_log_handles;
use pueue::network::message::*;
use pueue::state::SharedState;
use pueue::task::TaskStatus;

/// Invoked when calling `pueue clean`.
/// Remove all failed or done tasks from the state.
pub fn clean(state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    state.backup();
    let (matching, _) = state.tasks_in_statuses(vec![TaskStatus::Done], None);

    for task_id in &matching {
        if !state.is_task_removable(task_id, &[]) {
            continue;
        }
        let _ = state.tasks.remove(task_id).unwrap();
        clean_log_handles(*task_id, &state.settings.shared.pueue_directory);
    }

    state.save();

    create_success_message("All finished tasks have been removed")
}
