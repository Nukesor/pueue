use chrono::TimeDelta;
use comfy_table::{Cell, ContentArrangement, Row, Table, presets::UTF8_HORIZONTAL_ONLY};
use crossterm::style::Color;
use pueue_lib::{
    settings::Settings,
    task::{Task, TaskResult, TaskStatus},
};

use super::{OutputStyle, formatted_start_end, query::Rule, start_of_today};

/// This builder is responsible for determining which table columns should be displayed and
/// building a full [comfy_table] from a list of given [Task]s.
#[derive(Debug, Clone)]
pub struct TableBuilder<'a> {
    settings: &'a Settings,
    style: &'a OutputStyle,

    /// Whether the columns to be displayed are explicitly selected by the user.
    /// If that's the case, we won't do any automated checks whether columns should be displayed or
    /// not.
    selected_columns: bool,

    /// This following fields represent which columns should be displayed when executing
    /// `pueue status`. `true` for any column means that it'll be shown in the table.
    id: bool,
    status: bool,
    priority: bool,
    enqueue_at: bool,
    dependencies: bool,
    label: bool,
    command: bool,
    path: bool,
    start: bool,
    end: bool,
}

impl<'a> TableBuilder<'a> {
    pub fn new(settings: &'a Settings, style: &'a OutputStyle) -> Self {
        Self {
            settings,
            style,
            selected_columns: false,
            id: true,
            status: true,
            priority: false,
            enqueue_at: false,
            dependencies: false,
            label: false,
            command: true,
            path: true,
            start: true,
            end: true,
        }
    }

    pub fn build(mut self, tasks: &[Task]) -> Table {
        self.determine_special_columns(tasks);

        let mut table = Table::new();
        table
            .set_content_arrangement(ContentArrangement::Dynamic)
            .load_preset(UTF8_HORIZONTAL_ONLY)
            .set_header(self.build_header())
            .add_rows(self.build_task_rows(tasks));

        // Explicitly force styling, in case we aren't on a tty, but `--color=always` is set.
        if self.style.enabled {
            table.enforce_styling();
        }

        table
    }

    /// By default, several columns aren't shown until there's at least one task with relevant data.
    /// This function determines whether any of those columns should be shown.
    fn determine_special_columns(&mut self, tasks: &[Task]) {
        if self.selected_columns {
            return;
        }

        // Check whether there are any tasks with a non-default priority
        if tasks.iter().any(|task| task.priority != 0) {
            self.priority = true;
        }

        // Check whether there are any delayed tasks.
        let has_delayed_tasks = tasks.iter().any(|task| {
            matches!(
                task.status,
                TaskStatus::Stashed {
                    enqueue_at: Some(_)
                }
            )
        });
        if has_delayed_tasks {
            self.enqueue_at = true;
        }

        // Check whether there are any tasks with dependencies.
        if tasks.iter().any(|task| !task.dependencies.is_empty()) {
            self.dependencies = true;
        }

        // Check whether there are any tasks a label.
        if tasks.iter().any(|task| task.label.is_some()) {
            self.label = true;
        }
    }

    /// Take a list of given [pest] rules from our `crate::client::query::column_selection::apply`
    /// logic. Set the column visibility based on these rules.
    pub fn set_visibility_by_rules(&mut self, rules: &[Rule]) {
        // Don't change anything, if there're no rules
        if rules.is_empty() {
            return;
        }

        // First of all, make all columns invisible.
        self.id = false;
        self.status = false;
        self.priority = false;
        self.enqueue_at = false;
        self.dependencies = false;
        self.label = false;
        self.command = false;
        self.path = false;
        self.start = false;
        self.end = false;

        // Make sure we don't do any default column visibility checks of our own.
        self.selected_columns = true;

        for rule in rules {
            match rule {
                Rule::column_id => self.id = true,
                Rule::column_status => self.status = true,
                Rule::column_priority => self.priority = true,
                Rule::column_enqueue_at => self.enqueue_at = true,
                Rule::column_dependencies => self.dependencies = true,
                Rule::column_label => self.label = true,
                Rule::column_command => self.command = true,
                Rule::column_path => self.path = true,
                Rule::column_start => self.start = true,
                Rule::column_end => self.end = true,
                _ => (),
            }
        }
    }

    /// Build a header row based on the current selection of columns.
    fn build_header(&self) -> Row {
        let mut header = Vec::new();

        // Create table header row
        if self.id {
            header.push(Cell::new("Id"));
        }
        if self.status {
            header.push(Cell::new("Status"));
        }
        if self.priority {
            header.push(Cell::new("Prio"));
        }
        if self.enqueue_at {
            header.push(Cell::new("Enqueue At"));
        }
        if self.dependencies {
            header.push(Cell::new("Deps"));
        }
        if self.label {
            header.push(Cell::new("Label"));
        }
        if self.command {
            header.push(Cell::new("Command"));
        }
        if self.path {
            header.push(Cell::new("Path"));
        }
        if self.start {
            header.push(Cell::new("Start"));
        }
        if self.end {
            header.push(Cell::new("End"));
        }

        Row::from(header)
    }

    fn build_task_rows(&self, tasks: &[Task]) -> Vec<Row> {
        let mut rows = Vec::new();
        // Add rows one by one.
        for task in tasks.iter() {
            let mut row = Row::new();
            // Users can set a max height per row.
            if let Some(height) = self.settings.client.max_status_lines {
                row.max_height(height);
            }

            if self.id {
                row.add_cell(Cell::new(task.id));
            }

            if self.status {
                // Determine the human readable task status representation and the respective color.
                let status_string = task.status.to_string();
                let (status_text, color) = match &task.status {
                    TaskStatus::Running { .. } => (status_string, Color::Green),
                    TaskStatus::Paused { .. } | TaskStatus::Locked { .. } => {
                        (status_string, Color::White)
                    }
                    TaskStatus::Done { result, .. } => match result {
                        TaskResult::Success => (TaskResult::Success.to_string(), Color::Green),
                        TaskResult::DependencyFailed => {
                            ("Dependency failed".to_string(), Color::Red)
                        }
                        TaskResult::FailedToSpawn(_) => ("Failed to spawn".to_string(), Color::Red),
                        TaskResult::Failed(code) => (format!("Failed ({code})"), Color::Red),
                        _ => (result.to_string(), Color::Red),
                    },
                    _ => (status_string, Color::Yellow),
                };
                row.add_cell(self.style.styled_cell(status_text, Some(color), None));
            }

            if self.priority {
                row.add_cell(Cell::new(task.priority.to_string()));
            }

            if self.enqueue_at {
                if let TaskStatus::Stashed {
                    enqueue_at: Some(enqueue_at),
                } = task.status
                {
                    // Only show the date if the task is not supposed to be enqueued today.
                    let enqueue_today =
                        enqueue_at <= start_of_today() + TimeDelta::try_days(1).unwrap();
                    let formatted_enqueue_at = if enqueue_today {
                        enqueue_at.format(&self.settings.client.status_time_format)
                    } else {
                        enqueue_at.format(&self.settings.client.status_datetime_format)
                    };
                    row.add_cell(Cell::new(formatted_enqueue_at));
                } else {
                    row.add_cell(Cell::new(""));
                }
            }

            if self.dependencies {
                let text = task
                    .dependencies
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                row.add_cell(Cell::new(text));
            }

            if self.label {
                row.add_cell(Cell::new(task.label.as_deref().unwrap_or_default()));
            }

            // Add command and path.
            if self.command {
                if self.settings.client.show_expanded_aliases {
                    row.add_cell(Cell::new(&task.command));
                } else {
                    row.add_cell(Cell::new(&task.original_command));
                }
            }

            if self.path {
                row.add_cell(Cell::new(task.path.to_string_lossy()));
            }

            // Add start and end info
            let (start, end) = formatted_start_end(task, self.settings);
            if self.start {
                row.add_cell(Cell::new(start));
            }
            if self.end {
                row.add_cell(Cell::new(end));
            }

            rows.push(row);
        }

        rows
    }
}
