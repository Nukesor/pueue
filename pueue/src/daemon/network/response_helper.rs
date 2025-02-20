use pueue_lib::{
    Group, Response, Task,
    network::message::{create_failure_response, create_success_response},
    state::FilteredTasks,
};

use crate::daemon::internal_state::state::LockedState;

/// Check whether a given group exists. Return a failure message if it doesn't.
pub fn ensure_group_exists<'state>(
    state: &'state mut LockedState,
    group: &str,
) -> Result<&'state mut Group, Response> {
    let group_keys: Vec<String> = state.groups().keys().cloned().collect();
    if let Some(group) = state.groups_mut().get_mut(group) {
        return Ok(group);
    }

    Err(create_failure_response(format!(
        "Group {group} doesn't exists. Use one of these: {group_keys:?}",
    )))
}

/// Compile a response for an action that affect several given tasks.
/// That action can sometimes only succeed for a portion of the given tasks.
/// E.g. only running tasks can be killed.
///
/// That's why this helper exists, which determines for which tasks an action succeeds
/// and which tasks fail, based on a given `filter` criterion.
/// ```rs
/// task_ids = vec![1, 2, 4];
/// task_action_response_helper(
///     "Tasks are being killed",
///     task_ids.clone(),
///     Task::is_running,
///     &state,
/// ),
/// ```
pub fn task_action_response_helper<F>(
    message: &str,
    task_ids: Vec<usize>,
    filter: F,
    state: &LockedState,
) -> Response
where
    F: Fn(&Task) -> bool,
{
    // Get all matching/mismatching task_ids for all given ids and statuses.
    let filtered_tasks = state.filter_tasks(filter, Some(task_ids));

    compile_task_response(message, filtered_tasks)
}

/// Compile a response for instructions with multiple tasks ids.
/// A custom message will be combined with a text about all matching tasks
/// and possibly tasks for which the instruction cannot be executed.
pub fn compile_task_response(message: &str, filtered_tasks: FilteredTasks) -> Response {
    let matching_ids: Vec<String> = filtered_tasks
        .matching_ids
        .iter()
        .map(|id| id.to_string())
        .collect();
    let non_matching_ids: Vec<String> = filtered_tasks
        .non_matching_ids
        .iter()
        .map(|id| id.to_string())
        .collect();
    let matching_ids = matching_ids.join(", ");

    // We don't have any mismatching ids, return the simple message.
    if filtered_tasks.non_matching_ids.is_empty() {
        return create_success_response(format!("{message}: {matching_ids}"));
    }

    let mismatched_message = "The command failed for tasks";
    let mismatching_ids = non_matching_ids.join(", ");

    // All given ids are invalid.
    if matching_ids.is_empty() {
        return create_failure_response(format!("{mismatched_message}: {mismatching_ids}"));
    }

    // Some ids were valid, some were invalid.
    create_success_response(format!(
        "{message}: {matching_ids}\n{mismatched_message}: {mismatching_ids}",
    ))
}
