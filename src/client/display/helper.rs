use std::collections::BTreeMap;
use std::io::stdout;

use crossterm::style::{style, Attribute, Color};
use crossterm::tty::IsTty;

use pueue_lib::state::GroupStatus;
use pueue_lib::task::Task;

/// This is a simple small helper function with the purpose of easily styling text,
/// while also prevent styling if we're printing to a non-tty output.
/// If there's any kind of styling in the code, it should be done with the help of this function.
pub fn style_text<T: ToString>(
    text: T,
    color: Option<Color>,
    attribute: Option<Attribute>,
) -> String {
    let text = text.to_string();
    // No tty, we aren't allowed to do any styling
    if !stdout().is_tty() {
        return text;
    }

    let mut styled = style(text);
    if let Some(color) = color {
        styled = styled.with(color);
    }
    if let Some(attribute) = attribute {
        styled = styled.attribute(attribute);
    }

    styled.to_string()
}

/// By default, several columns aren't shown until there's actually some data to display.
/// This function determines, which of those columns actually need to be shown.
pub fn has_special_columns(tasks: &BTreeMap<usize, Task>) -> (bool, bool, bool) {
    // Check whether there are any delayed tasks.
    let has_delayed_tasks = tasks.iter().any(|(_id, task)| task.enqueue_at.is_some());

    // Check whether there are any tasks with dependencies.
    let has_dependencies = tasks
        .iter()
        .any(|(_id, task)| !task.dependencies.is_empty());

    // Check whether there are any tasks a label.
    let has_labels = tasks.iter().any(|(_id, task)| task.label.is_some());

    (has_delayed_tasks, has_dependencies, has_labels)
}

/// Return a nicely formatted headline that's displayed above group tables
pub fn get_group_headline(name: &str, status: &GroupStatus, parallel: usize) -> String {
    // Style group name
    let name = style(format!("Group \"{}\"", name)).attribute(Attribute::Bold);

    // Print the current state of the group.
    let status = match status {
        GroupStatus::Running => style_text("running", Some(Color::Green), None),
        GroupStatus::Paused => style_text("paused", Some(Color::Yellow), None),
    };

    format!("{} ({} parallel): {}", name, parallel, status)
}

/// Sort given tasks by their groups
/// This is needed to print a table for each group
pub fn sort_tasks_by_group(
    tasks: &BTreeMap<usize, Task>,
) -> BTreeMap<String, BTreeMap<usize, Task>> {
    // We use a BTreeMap, since groups should be ordered alphabetically by their name
    let mut sorted_task_groups = BTreeMap::new();
    for (id, task) in tasks.iter() {
        if !sorted_task_groups.contains_key(&task.group) {
            sorted_task_groups.insert(task.group.clone(), BTreeMap::new());
        }
        sorted_task_groups
            .get_mut(&task.group)
            .unwrap()
            .insert(*id, task.clone());
    }

    sorted_task_groups
}
