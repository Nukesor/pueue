use std::collections::BTreeMap;

use chrono::{DateTime, Local, LocalResult};
use pueue_lib::{settings::Settings, task::Task};

/// Try to get the start of the current date to the best of our abilities.
/// Throw an error, if we can't.
pub fn start_of_today() -> DateTime<Local> {
    let result = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("Failed to find start of today.")
        .and_local_timezone(Local);

    // Try to get the start of the current date.
    // If there's no unambiguous result for today's midnight, we pick the first value as a backup.
    match result {
        LocalResult::None => panic!("Failed to find start of today."),
        LocalResult::Single(today) => today,
        LocalResult::Ambiguous(today, _) => today,
    }
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

/// Returns the formatted `start` and `end` text for a given task.
///
/// 1. If the start || end is today, skip the date.
/// 2. Otherwise show the date in both.
///
/// If the task doesn't have a start and/or end yet, an empty string will be returned
/// for the respective field.
pub fn formatted_start_end(task: &Task, settings: &Settings) -> (String, String) {
    let (start, end) = task.start_and_end();

    // If the task didn't start yet, just return two empty strings.
    let start = match start {
        Some(start) => start,
        None => return ("".into(), "".into()),
    };

    // If the task started today, just show the time.
    // Otherwise show the full date and time.
    let started_today = start >= start_of_today();
    let formatted_start = if started_today {
        start
            .format(&settings.client.status_time_format)
            .to_string()
    } else {
        start
            .format(&settings.client.status_datetime_format)
            .to_string()
    };

    // If the task didn't finish yet, only return the formatted start.
    let end = match end {
        Some(end) => end,
        None => return (formatted_start, "".into()),
    };

    // If the task ended today we only show the time.
    // In all other circumstances, we show the full date.
    let finished_today = end >= start_of_today();
    let formatted_end = if finished_today {
        end.format(&settings.client.status_time_format).to_string()
    } else {
        end.format(&settings.client.status_datetime_format)
            .to_string()
    };

    (formatted_start, formatted_end)
}
