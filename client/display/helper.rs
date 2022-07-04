use std::collections::BTreeMap;

use crossterm::style::Attribute;

use pueue_lib::state::{Group, GroupStatus};
use pueue_lib::task::{Task, TaskStatus};

use super::OutputStyle;

/// By default, several columns aren't shown until there's actually some data to display.
/// This function determines, which of those columns actually need to be shown.
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

/// Return a nicely formatted headline that's displayed above group tables
pub fn get_group_headline(name: &str, group: &Group, style: &OutputStyle) -> String {
    // Style group name
    let name = style.style_text(format!("Group \"{}\"", name), None, Some(Attribute::Bold));

    // Print the current state of the group.
    let status = match group.status {
        GroupStatus::Running => style.style_text("running", Some(style.green()), None),
        GroupStatus::Paused => style.style_text("paused", Some(style.yellow()), None),
    };

    format!("{} ({} parallel): {}", name, group.parallel_tasks, status)
}

/// Sort given tasks by their groups
/// This is needed to print a table for each group
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
