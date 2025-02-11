use std::{
    collections::BTreeMap,
    io::{self, prelude::*},
};

use chrono::{DateTime, Local, LocalResult};
use pueue_lib::{
    settings::Settings,
    state::{State, PUEUE_DEFAULT_GROUP},
    task::Task,
};

use crate::{
    client::{
        client::Client, commands::get_state, display_helper::get_group_headline, style::OutputStyle,
    },
    internal_prelude::*,
};

mod query;
mod table_builder;

use query::apply_query;
use table_builder::TableBuilder;

/// Simply request and print the state.
pub async fn state(
    client: &mut Client,
    query: Vec<String>,
    json: bool,
    group: Option<String>,
) -> Result<()> {
    let state = get_state(client).await?;
    let tasks = state.tasks.values().cloned().collect();

    let output = print_state(
        state,
        tasks,
        &client.style,
        &client.settings,
        json,
        group,
        Some(query),
    )?;
    println!("{output}");

    Ok(())
}

/// This function tries to read a map or list of JSON serialized [Task]s from `stdin`.
/// The tasks will then get deserialized and displayed as a normal `status` command.
/// The current group information is pulled from the daemon in a new `status` call.
pub async fn format_state(client: &mut Client, group: Option<String>) -> Result<()> {
    // Read the raw input to a buffer
    let mut stdin = io::stdin();
    let mut buffer = Vec::new();
    stdin
        .read_to_end(&mut buffer)
        .context("Failed to read json from stdin.")?;

    // Convert it to a valid utf8 stream. If this fails, it cannot be valid JSON.
    let json = String::from_utf8(buffer).context("Failed to convert stdin input to UTF8")?;

    // Try to deserialize the input as a map of tasks first.
    // If this doesn't work, try a list of tasks.
    let map_deserialize = serde_json::from_str::<BTreeMap<usize, Task>>(&json);

    let tasks: Vec<Task> = if let Ok(map) = map_deserialize {
        map.into_values().collect()
    } else {
        serde_json::from_str(&json).context("Failed to deserialize from JSON input.")?
    };

    let state = get_state(client)
        .await
        .context("Failed to get the current state from daemon")?;

    let output = print_state(
        state,
        tasks,
        &client.style,
        &client.settings,
        false,
        group,
        None,
    )?;
    print!("{output}");

    Ok(())
}

/// Get the output for the state of the daemon in a nicely formatted table.
/// If there are multiple groups, each group with a task will have its own table.
///
/// We pass the tasks as a separate parameter and as a list.
/// This allows us to print the tasks in the order passed to the `format-status` subcommand.
fn print_state(
    mut state: State,
    mut tasks: Vec<Task>,
    style: &OutputStyle,
    settings: &Settings,
    json: bool,
    group: Option<String>,
    query: Option<Vec<String>>,
) -> Result<String> {
    let mut output = String::new();

    let mut table_builder = TableBuilder::new(settings, style);

    if let Some(query) = &query {
        let query_result = apply_query(&query.join(" "), &group)?;
        table_builder.set_visibility_by_rules(&query_result.selected_columns);
        tasks = query_result.apply_filters(tasks);
        tasks = query_result.order_tasks(tasks);
        tasks = query_result.limit_tasks(tasks);
    }

    // If the json flag is specified, print the state as json and exit.
    if json {
        if query.is_some() {
            state.tasks = tasks.into_iter().map(|task| (task.id, task)).collect();
        }
        output.push_str(&serde_json::to_string(&state).unwrap());
        return Ok(output);
    }

    if let Some(group) = group {
        print_single_group(state, tasks, style, group, table_builder, &mut output);
        return Ok(output);
    }

    print_all_groups(state, tasks, style, table_builder, &mut output);

    Ok(output)
}

/// The user requested only a single group to be displayed.
///
/// Print this group or show an error if this group doesn't exist.
fn print_single_group(
    state: State,
    tasks: Vec<Task>,
    style: &OutputStyle,
    group_name: String,
    table_builder: TableBuilder,
    output: &mut String,
) {
    // Sort all tasks by their respective group;
    let mut sorted_tasks = sort_tasks_by_group(tasks);

    let Some(group) = state.groups.get(&group_name) else {
        eprintln!("There exists no group \"{group_name}\"");
        return;
    };

    // Only a single group is requested. Print that group and return.
    let tasks = sorted_tasks.entry(group_name.clone()).or_default();
    let headline = get_group_headline(&group_name, group, style);
    output.push_str(&headline);

    // Show a message if the requested group doesn't have any tasks.
    if tasks.is_empty() {
        output.push_str(&format!(
            "\nTask list is empty. Add tasks with `pueue add -g {group_name} -- [cmd]`"
        ));
        return;
    }

    let table = table_builder.build(tasks);
    output.push_str(&format!("\n{table}"));
}

/// Print all groups. All tasks will be shown in the table of their assigned group.
///
/// This will create multiple tables, one table for each group.
fn print_all_groups(
    state: State,
    tasks: Vec<Task>,
    style: &OutputStyle,
    table_builder: TableBuilder,
    output: &mut String,
) {
    // Early exit and hint if there are no tasks in the queue
    // Print the state of the default group anyway, since this is information one wants to
    // see most of the time anyway.
    if state.tasks.is_empty() {
        let headline = get_group_headline(
            PUEUE_DEFAULT_GROUP,
            state.groups.get(PUEUE_DEFAULT_GROUP).unwrap(),
            style,
        );
        output.push_str(&format!("{headline}\n"));
        output.push_str("\nTask list is empty. Add tasks with `pueue add -- [cmd]`");
        return;
    }

    // Sort all tasks by their respective group;
    let sorted_tasks = sort_tasks_by_group(tasks);

    // Always print the default queue at the very top, if no specific group is requested.
    if sorted_tasks.contains_key(PUEUE_DEFAULT_GROUP) {
        let tasks = sorted_tasks.get(PUEUE_DEFAULT_GROUP).unwrap();
        let headline = get_group_headline(
            PUEUE_DEFAULT_GROUP,
            state.groups.get(PUEUE_DEFAULT_GROUP).unwrap(),
            style,
        );
        output.push_str(&headline);
        let table = table_builder.clone().build(tasks);
        output.push_str(&format!("\n{table}"));

        // Add a newline if there are further groups to be printed
        if sorted_tasks.len() > 1 {
            output.push('\n');
        }
    }

    // Print a table for every other group that has any tasks
    let mut sorted_iter = sorted_tasks.iter().peekable();
    while let Some((group, tasks)) = sorted_iter.next() {
        // We always want to print the default group at the very top.
        // That's why we print it before this loop and skip it in here.
        if group.eq(PUEUE_DEFAULT_GROUP) {
            continue;
        }

        let headline = get_group_headline(group, state.groups.get(group).unwrap(), style);
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(&headline);
        let table = table_builder.clone().build(tasks);
        output.push_str(&format!("\n{table}"));

        // Add a newline between groups
        if sorted_iter.peek().is_some() {
            output.push('\n');
        }
    }
}

/// Try to get the start of the current date to the best of our abilities.
/// Throw an error, if we can't.
fn start_of_today() -> DateTime<Local> {
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
fn sort_tasks_by_group(tasks: Vec<Task>) -> BTreeMap<String, Vec<Task>> {
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
fn formatted_start_end(task: &Task, settings: &Settings) -> (String, String) {
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
