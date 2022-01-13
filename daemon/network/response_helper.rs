use std::sync::MutexGuard;

use pueue_lib::network::message::{create_failure_message, create_success_message, Message};
use pueue_lib::state::{Group, State};
use pueue_lib::task::Task;

use crate::state_helper::LockedState;

/// Check whether the given group exists. Return an failure message if it doesn't.
pub fn ensure_group_exists<'state>(
    state: &'state mut LockedState,
    group: &str,
) -> Result<&'state mut Group, Message> {
    let group_keys: Vec<String> = state.groups.keys().cloned().collect();
    if let Some(group) = state.groups.get_mut(group) {
        return Ok(group);
    }

    Err(create_failure_message(format!(
        "Group {group} doesn't exists. Use one of these: {group_keys:?}",
    )))
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
    let mismatching_ids: Vec<String> = mismatching.iter().map(|id| id.to_string()).collect();
    let matching_ids = matching.join(", ");

    // We don't have any mismatching ids, return the simple message.
    if mismatching.is_empty() {
        return create_success_message(format!("{message}: {matching_ids}"));
    }

    let mismatched_message = "The command failed for tasks";
    let mismatching_ids = mismatching_ids.join(", ");

    // All given ids are invalid.
    if matching.is_empty() {
        return create_failure_message(format!("{mismatched_message}: {mismatching_ids}"));
    }

    // Some ids were valid, some were invalid.
    create_success_message(format!(
        "{message}: {matching_ids}\n{mismatched_message}: {mismatching_ids}",
    ))
}
