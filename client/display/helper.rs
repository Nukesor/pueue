use std::collections::BTreeMap;

use chrono::Local;

use pueue_lib::{settings::Settings, task::Task};

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

/// Returns the formatted `start` and `end` text for a given task.
///
/// 1. If the start || end is today, skip the date.
/// 2. Otherwise show the date in both.
///
/// If the task doesn't have a start and/or end yet, an empty string will be returned
/// for the respective field.
pub fn formatted_start_end(task: &Task, settings: &Settings) -> (String, String) {
    // Get the start time.
    // If the task didn't start yet, just return two empty strings.
    let start = match task.start {
        Some(start) => start,
        None => return ("".into(), "".into()),
    };

    // If the task started today, just show the time.
    // Otherwise show the full date and time.
    let started_today = start >= Local::today().and_hms(0, 0, 0);
    let formatted_start = if started_today {
        start
            .format(&settings.client.status_time_format)
            .to_string()
    } else {
        start
            .format(&settings.client.status_datetime_format)
            .to_string()
    };

    // Get finish time, if already set. Otherwise only return the formatted start.
    let end = match task.end {
        Some(end) => end,
        None => return (formatted_start, "".into()),
    };

    // If the task ended today we only show the time.
    // In all other circumstances, we show the full date.
    let finished_today = end >= Local::today().and_hms(0, 0, 0);
    let formatted_end = if finished_today {
        end.format(&settings.client.status_time_format).to_string()
    } else {
        end.format(&settings.client.status_datetime_format)
            .to_string()
    };

    (formatted_start, formatted_end)
}
