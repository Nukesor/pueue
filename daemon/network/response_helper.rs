use std::sync::MutexGuard;

use pueue_lib::network::message::{create_failure_message, create_success_message, Message};
use pueue_lib::state::State;
use pueue_lib::task::Task;

/// Check whether the given group exists. Return an failure message if it doesn't.
pub fn ensure_group_exists(state: &MutexGuard<State>, group: &str) -> Result<(), Message> {
    if !state.groups.contains_key(group) {
        return Err(create_failure_message(format!(
            "Group {} doesn't exists. Use one of these: {:?}",
            group,
            state.groups.keys()
        )));
    }

    Ok(())
}

/// Compile a response for actions that affect several given tasks.
/// These actions can sometimes only succeed for a part of the given tasks.
///
/// That's why this helper exists, which determines based on a given criterion `filter`
/// for which tasks the action succeeded and which tasks failed.
pub fn task_action_response_helper<F>(
    message: &str,
    task_ids: Vec<usize>,
    filter: F,
    state: &MutexGuard<State>,
) -> Message
where
    F: Fn(&Task) -> bool,
{
    // Get all matching/mismatching task_ids for all given ids and statuses.
    let (matching, mismatching) = state.filter_tasks(filter, Some(task_ids));

    compile_task_response(message, matching, mismatching)
}

/// Compile a response for instructions with multiple tasks ids
/// A custom message will be combined with a text about all matching tasks
/// and possibly tasks for which the instruction cannot be executed.
pub fn compile_task_response(
    message: &str,
    matching: Vec<usize>,
    mismatching: Vec<usize>,
) -> Message {
    let matching: Vec<String> = matching.iter().map(|id| id.to_string()).collect();
    let mismatching: Vec<String> = mismatching.iter().map(|id| id.to_string()).collect();
    let matching_string = matching.join(", ");

    // We don't have any mismatching ids, return the simple message.
    if mismatching.is_empty() {
        return create_success_message(format!("{}: {}", message, matching_string));
    }

    let mismatched_message = "The command failed for tasks";
    let mismatching_string = mismatching.join(", ");

    // All given ids are invalid.
    if matching.is_empty() {
        return create_failure_message(format!("{}: {}", mismatched_message, mismatching_string));
    }

    // Some ids were valid, some were invalid.
    create_success_message(format!(
        "{}: {}\n{}: {}",
        message, matching_string, mismatched_message, mismatching_string
    ))
}
