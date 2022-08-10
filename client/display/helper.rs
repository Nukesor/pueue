use std::collections::BTreeMap;

use pueue_lib::task::{Task, TaskStatus};

/// This is a helper function for working with tables when calling `pueue status`.
///
/// By default, several columns aren't shown until there's at least one task with relevant data.
/// This function determines whether any of those columns should be shown.
pub fn has_special_columns(tasks: &[Task]) -> (bool, bool, bool) {
    // Check whether there are any delayed tasks.
    let has_delayed_tasks = tasks.iter().any(|task| {
        matches!(
            task.status,
            TaskStatus::Stashed {
                enqueue_at: Some(_)
            }
        )
    });

    // Check whether there are any tasks with dependencies.
    let has_dependencies = tasks.iter().any(|task| !task.dependencies.is_empty());

    // Check whether there are any tasks a label.
    let has_labels = tasks.iter().any(|task| task.label.is_some());

    (has_delayed_tasks, has_dependencies, has_labels)
}

/// Sort given tasks by their groups.
/// This is needed to print a table for each group.
pub fn sort_tasks_by_group(tasks: Vec<Task>) -> BTreeMap<String, Vec<Task>> {
    // We use a BTreeMap, since groups should be ordered alphabetically by their name
    let mut sorted_task_groups = BTreeMap::new();
    for task in tasks.into_iter() {
        if !sorted_task_groups.contains_key(&task.group) {
            sorted_task_groups.insert(task.group.clone(), Vec::new());
        }
        sorted_task_groups.get_mut(&task.group).unwrap().push(task);
    }

    sorted_task_groups
}
