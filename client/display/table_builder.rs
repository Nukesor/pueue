use chrono::{Duration, Local};
use comfy_table::presets::UTF8_HORIZONTAL_ONLY;
use comfy_table::*;

use pueue_lib::settings::Settings;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use super::helper::formatted_start_end;
use super::OutputStyle;

/// This builder is responsible for determining which table columns should be displayed and
/// building a full [comfy_table] from a list of given [Task]s.
#[derive(Debug, Clone)]
pub struct TableBuilder<'a> {
    settings: &'a Settings,
    style: &'a OutputStyle,

    /// This following fields represent which columns should be displayed when executing
    /// `pueue status`. `true` for any column means that it'll be shown in the table.
    id: bool,
    status: bool,
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

            id: true,
            status: true,
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
                row.add_cell(Cell::new(&task.id));
            }

            if self.status {
                // Determine the human readable task status representation and the respective color.
                let status_string = task.status.to_string();
                let (status_text, color) = match &task.status {
                    TaskStatus::Running => (status_string, Color::Green),
                    TaskStatus::Paused | TaskStatus::Locked => (status_string, Color::White),
                    TaskStatus::Done(result) => match result {
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

            if self.enqueue_at {
                if let TaskStatus::Stashed {
                    enqueue_at: Some(enqueue_at),
                } = task.status
                {
                    // Only show the date if the task is not supposed to be enqueued today.
                    let enqueue_today =
                        enqueue_at <= Local::today().and_hms(0, 0, 0) + Duration::days(1);
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
                row.add_cell(Cell::new(&task.label.as_deref().unwrap_or_default()));
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
                row.add_cell(Cell::new(&task.path.to_string_lossy()));
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
