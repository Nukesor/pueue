use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;

use crate::network::response_helper::*;
use crate::state_helper::save_settings;

/// Set the parallel tasks for either a specific group or the global default.
pub fn set_parallel_tasks(message: ParallelMessage, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    let group = match ensure_group_exists(&mut state, &message.group) {
        Ok(group) => group,
        Err(message) => return message,
    };

    group.parallel_tasks = message.parallel_tasks;

    if let Err(error) = save_settings(&state) {
        return create_failure_message(format!("Failed while saving the config file: {}", error));
    }

    create_success_message(format!(
        "Parallel tasks setting for group \"{}\" adjusted",
        &message.group
    ))
}
