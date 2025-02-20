use pueue_lib::{GroupStatus, Settings, failure_msg, network::message::*};

use crate::daemon::{internal_state::SharedState, process_handler};

/// Invoked when calling `pueue reset`.
/// Kill all children by using the `kill` function.
/// Set the full_reset flag, which will prevent new tasks from being spawned.
pub fn reset(settings: &Settings, state: &SharedState, message: ResetRequest) -> Response {
    let mut state = state.lock().unwrap();

    match message.target {
        ResetTarget::All => {
            // Mark all groups to be reset and kill all tasks
            for (_name, group) in state.groups_mut().iter_mut() {
                group.status = GroupStatus::Reset;
            }
            process_handler::kill::kill(settings, &mut state, TaskSelection::All, false, None);
        }
        ResetTarget::Groups(groups) => {
            // First up, check whether we actually have all requested groups.
            for name in groups.iter() {
                let group = state.groups().get(name);
                if group.is_none() {
                    return failure_msg!("Group '{name}' doesn't exist.");
                }
            }

            // Mark all groups to be reset and kill its tasks
            for name in groups.iter() {
                let group = state.groups_mut().get_mut(name).unwrap();
                group.status = GroupStatus::Reset;

                process_handler::kill::kill(
                    settings,
                    &mut state,
                    TaskSelection::Group(name.to_string()),
                    false,
                    None,
                );
            }
        }
    }
    create_success_response("Everything is being reset right now.")
}
