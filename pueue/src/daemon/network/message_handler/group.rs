use std::collections::BTreeMap;

use pueue_lib::{
    failure_msg, network::message::*, settings::Settings, state::PUEUE_DEFAULT_GROUP, success_msg,
};

use crate::{
    daemon::{
        internal_state::SharedState,
        network::{message_handler::ok_or_failure_message, response_helper::ensure_group_exists},
        process_handler::initiate_shutdown,
    },
    ok_or_save_state_failure,
};

/// Invoked on `pueue groups`.
/// Manage groups.
/// - Show groups
/// - Add group
/// - Remove group
pub fn group(settings: &Settings, state: &SharedState, message: GroupRequest) -> Response {
    let mut state = state.lock().unwrap();

    match message {
        GroupRequest::List => {
            // Return information about all groups to the client.
            GroupResponse {
                groups: state.groups().clone(),
            }
            .into()
        }
        GroupRequest::Add {
            name,
            parallel_tasks,
        } => {
            if state.groups().contains_key(&name) {
                return failure_msg!("Group \"{name}\" already exists");
            }

            let group = state.create_group(&name);
            if let Some(parallel_tasks) = parallel_tasks {
                group.parallel_tasks = parallel_tasks;
            }
            // Create the worker pool.
            state.children.0.insert(name.clone(), BTreeMap::new());

            // Persist the state.
            ok_or_save_state_failure!(state.save(settings));

            success_msg!("New group \"{name}\" has been created")
        }
        GroupRequest::Remove(group) => {
            if let Err(message) = ensure_group_exists(&mut state, &group) {
                return message;
            }

            if group == PUEUE_DEFAULT_GROUP {
                return failure_msg!("You cannot delete the default group");
            }

            // Make sure there are no tasks in that group.
            if state.tasks().iter().any(|(_, task)| task.group == group) {
                return failure_msg!("You cannot remove a group, if there're still tasks in it.");
            }

            // Make sure the worker pool exists and is empty.
            // There shouldn't be any children, if there are no tasks in this group.
            // Those are critical errors, as they indicate desynchronization inside our
            // internal datastructures, which is really bad.
            if let Some(pool) = state.children.0.get(&group) {
                if !pool.is_empty() {
                    initiate_shutdown(settings, &mut state, ShutdownRequest::Emergency);
                    return failure_msg!(
                        "Encountered a non-empty worker pool, while removing a group. This is a critical error. Please report this bug."
                    );
                }
            } else {
                initiate_shutdown(settings, &mut state, ShutdownRequest::Emergency);
                return failure_msg!(
                    "Encountered an group without an worker pool, while removing a group. This is a critical error. Please report this bug."
                );
            }

            if let Err(error) = state.remove_group(&group) {
                return failure_msg!("Error while removing group: \"{error}\"");
            }

            // Actually remove the worker pool.
            state.children.0.remove(&group);

            // Persist the state.
            ok_or_save_state_failure!(state.save(settings));

            success_msg!("Group \"{group}\" has been removed")
        }
    }
}
