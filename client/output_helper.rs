use ::comfy_table::*;
use ::crossterm::style::style;
use ::std::collections::BTreeMap;

use ::pueue::state::State;
use ::pueue::task::Task;

pub fn has_special_columns(tasks: &BTreeMap<usize, Task>) -> (bool, bool) {
    // Check whether there are any delayed tasks.
    // In case there are, we need to add another column to the table.
    let has_delayed_tasks = tasks.iter().any(|(_id, task)| task.enqueue_at.is_some());

    // Check whether there are any tasks with dependencies.
    // In case there are, we need to add another column to the table.
    let has_dependencies = tasks
        .iter()
        .any(|(_id, task)| !task.dependencies.is_empty());

    return (has_delayed_tasks, has_dependencies);
}

/// Return a nicely formatted headline that's displayed at the start of `pueue status`
pub fn get_daemon_headline(state: &State) -> String {
    // Print the current daemon state.
    let daemon_status_text = if state.running {
        style("running").with(Color::Green)
    } else {
        style("paused").with(Color::Yellow)
    };
    let parallel = state.settings.daemon.default_parallel_tasks;
    format!(
        "{} ({} parallel): {}",
        style("Daemon status").attribute(Attribute::Bold),
        parallel,
        daemon_status_text
    )
}

/// Return a nicely formatted headline that's displayed above group tables
pub fn get_group_headline(group: &String, state: &State) -> String {
    // Group name
    let group_text = style(format!("Group \"{}\"", group)).attribute(Attribute::Bold);

    let parallel = state.settings.daemon.groups.get(group).unwrap();

    // Print the current state of the group.
    if *state.groups.get(group).unwrap() {
        format!(
            "{} ({} parallel): {}",
            group_text,
            parallel,
            style("running").with(Color::Green)
        )
    } else {
        format!(
            "{} ({} parallel): {}",
            group_text,
            parallel,
            style("paused").with(Color::Yellow)
        )
    }
}

/// Get all tasks that aren't assigned to a group
/// Those tasks are displayed first.
pub fn get_default_tasks(tasks: &BTreeMap<usize, Task>) -> BTreeMap<usize, Task> {
    let mut default_tasks = BTreeMap::new();
    for (id, task) in tasks.iter() {
        if task.group.is_none() {
            default_tasks.insert(*id, task.clone());
        }
    }

    default_tasks
}

/// Sort given tasks by their groups
/// This is needed to print a table for each group
pub fn sort_tasks_by_group(
    tasks: &BTreeMap<usize, Task>,
) -> BTreeMap<String, BTreeMap<usize, Task>> {
    // We use a BTreeMap, since groups should be ordered alphabetically by their name
    let mut sorted_task_groups = BTreeMap::new();
    for (id, task) in tasks.iter() {
        if let Some(group) = &task.group {
            if !sorted_task_groups.contains_key(group) {
                sorted_task_groups.insert(group.clone(), BTreeMap::new());
            }
            sorted_task_groups
                .get_mut(group)
                .unwrap()
                .insert(*id, task.clone());
        }
    }

    sorted_task_groups
}
