use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;

use crate::network::response_helper::*;

/// Set the parallel tasks for a specific group.
pub fn set_parallel_tasks(message: ParallelMessage, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    let group = match ensure_group_exists(&mut state, &message.group) {
        Ok(group) => group,
        Err(message) => return message,
    };

    group.parallel_tasks = message.parallel_tasks;

    create_success_message(format!(
        "Parallel tasks setting for group \"{}\" adjusted",
        &message.group
    ))
}
