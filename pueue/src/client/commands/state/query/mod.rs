// Clippy generates a false-positive for an empty generated docstring in the query parser code.
#![allow(clippy::empty_docs)]
use chrono::prelude::*;
use pest::Parser;
use pest_derive::Parser;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use crate::internal_prelude::*;

mod column_selection;
mod filters;
mod limit;
mod order_by;

use limit::Limit;
use order_by::Direction;

/// See the pest docs on how this derive macro works and how to use pest:
/// https://docs.rs/pest/latest/pest/
#[derive(Parser)]
#[grammar = "./src/client/commands/state/query/syntax.pest"]
struct QueryParser;

type FilterFunction = dyn Fn(&Task) -> bool;

/// All applicable information that has been extracted from the query.
#[derive(Default)]
pub struct QueryResult {
    /// Filter results for a single group.
    group: Option<String>,

    /// The list of selected columns based.
    pub selected_columns: Vec<Rule>,

    /// A list of filter functions that should be applied to the list of tasks.
    filters: Vec<Box<FilterFunction>>,

    /// A list of filter functions that should be applied to the list of tasks.
    order_by: Option<(Rule, Direction)>,

    /// Limit
    limit: Option<(Limit, usize)>,
}

impl QueryResult {
    /// Take a list of tasks and apply all filters to it.
    pub fn apply_filters(&self, tasks: Vec<Task>) -> Vec<Task> {
        let mut iter = tasks.into_iter();

        // If requested, only look at tasks of a specific group.
        if let Some(group) = &self.group {
            iter = iter
                .filter(|task| task.group == *group)
                .collect::<Vec<Task>>()
                .into_iter();
        }

        for filter in self.filters.iter() {
            iter = iter.filter(filter).collect::<Vec<Task>>().into_iter();
        }
        iter.collect()
    }

    /// Take a list of tasks and apply all filters to it.
    pub fn order_tasks(&self, mut tasks: Vec<Task>) -> Vec<Task> {
        // Only apply ordering if it was requested.
        let Some((column, direction)) = &self.order_by else {
            return tasks;
        };

        // Sort the tasks by the specified column.
        tasks.sort_by(|task1, task2| match column {
            Rule::column_id => task1.id.cmp(&task2.id),
            Rule::column_status => {
                /// Rank a task status to allow ordering by status.
                /// Returns a u8 based on the expected
                fn rank_status(task: &Task) -> u8 {
                    match &task.status {
                        TaskStatus::Stashed { .. } => 0,
                        TaskStatus::Locked { .. } => 1,
                        TaskStatus::Queued { .. } => 2,
                        TaskStatus::Paused { .. } => 3,
                        TaskStatus::Running { .. } => 4,
                        TaskStatus::Done { result, .. } => match result {
                            TaskResult::Success => 6,
                            _ => 5,
                        },
                    }
                }

                rank_status(task1).cmp(&rank_status(task2))
            }
            Rule::column_label => task1.label.cmp(&task2.label),
            Rule::column_command => task1.command.cmp(&task2.command),
            Rule::column_path => task1.path.cmp(&task2.path),
            Rule::column_enqueue_at => {
                fn enqueue_date(task: &Task) -> DateTime<Local> {
                    match &task.status {
                        TaskStatus::Queued { enqueued_at, .. }
                        | TaskStatus::Running { enqueued_at, .. }
                        | TaskStatus::Paused { enqueued_at, .. }
                        | TaskStatus::Done { enqueued_at, .. }
                        | TaskStatus::Stashed {
                            enqueue_at: Some(enqueued_at),
                            ..
                        } => *enqueued_at,
                        // considered far in the future when no explicit date:
                        _ => DateTime::<Utc>::MAX_UTC.into(),
                    }
                }

                enqueue_date(task1).cmp(&enqueue_date(task2))
            }
            Rule::column_start => {
                let (start1, _) = task1.start_and_end();
                let (start2, _) = task2.start_and_end();
                start1.cmp(&start2)
            }
            Rule::column_end => {
                let (_, end1) = task1.start_and_end();
                let (_, end2) = task2.start_and_end();
                end1.cmp(&end2)
            }
            _ => std::cmp::Ordering::Less,
        });

        // Reverse the order, if we're in ordering by descending order.
        if let Direction::Descending = direction {
            tasks.reverse();
        }

        tasks
    }

    /// Take a list of tasks and apply all filters to it.
    pub fn limit_tasks(&self, tasks: Vec<Task>) -> Vec<Task> {
        // Only apply limits if it was requested.
        let Some((direction, count)) = &self.limit else {
            return tasks;
        };

        // Don't do anything if:
        // - we don't have to limit
        // - the limit is invalid
        if tasks.len() <= *count || *count == 0 {
            return tasks;
        }

        match direction {
            Limit::First => tasks[0..*count].to_vec(),
            Limit::Last => tasks[(tasks.len() - count)..].to_vec(),
        }
    }
}

/// Take a given `pueue status QUERY` and apply it to all components that're involved in the
/// `pueue status` process:
///
/// - TableBuilder: The component responsible for building the table and determining which columns
///   should or need to be displayed. A `columns [columns]` statement will define the set of visible
///   columns.
pub fn apply_query(query: &str, group: &Option<String>) -> Result<QueryResult> {
    let mut parsed = QueryParser::parse(Rule::query, query).context("Failed to parse query")?;

    let mut query_result = QueryResult {
        group: group.clone(),
        ..Default::default()
    };

    // Expect there to be exactly one pair for the full query.
    // Return early if we got an empty query.
    let Some(parsed) = parsed.next() else {
        return Ok(query_result);
    };

    // Make sure we really got a query.
    if parsed.as_rule() != Rule::query {
        bail!("Expected a valid query");
    }

    // Get the sections of the query
    let sections = parsed.into_inner();
    // Go through each section and handle it accordingly
    for section in sections {
        // The `columns=[columns]` section
        // E.g. `columns=id,status,start,end`
        match section.as_rule() {
            Rule::column_selection => column_selection::apply(section, &mut query_result)?,
            Rule::datetime_filter => filters::datetime(section, &mut query_result)?,
            Rule::label_filter => filters::label(section, &mut query_result)?,
            Rule::command_filter => filters::command(section, &mut query_result)?,
            Rule::status_filter => filters::status(section, &mut query_result)?,
            Rule::order_by_condition => order_by::order_by(section, &mut query_result)?,
            Rule::limit_condition => limit::limit(section, &mut query_result)?,
            _ => (),
        }
    }

    Ok(query_result)
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, path::PathBuf};

    use assert_matches::assert_matches;
    use chrono::{Local, TimeZone};
    use pretty_assertions::assert_eq;
    use pueue_lib::{
        state::PUEUE_DEFAULT_GROUP,
        task::{Task, TaskResult, TaskStatus},
    };
    use rstest::rstest;

    use super::{apply_query, Rule};
    use crate::internal_prelude::*;

    const TEST_COMMAND_SLEEP: &str = "sleep 60";
    const TEST_COMMAND_HELLO: &str = "echo Hello Pueue";

    /// A small helper function to reduce a bit of boilerplate.
    pub fn build_task() -> Task {
        Task::new(
            TEST_COMMAND_SLEEP.to_owned(),
            PathBuf::from("/tmp"),
            HashMap::new(),
            PUEUE_DEFAULT_GROUP.to_owned(),
            TaskStatus::Queued {
                enqueued_at: Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap(),
            },
            Vec::new(),
            0,
            None,
        )
    }

    /// Build a list of some pre-build tasks that are used to test.
    pub fn test_tasks() -> Vec<Task> {
        let mut tasks = Vec::new();

        // Failed task
        let mut failed = build_task();
        failed.id = 0;
        failed.status = TaskStatus::Done {
            result: TaskResult::Failed(255),
            enqueued_at: Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap(),
            start: Local.with_ymd_and_hms(2022, 1, 10, 10, 5, 0).unwrap(),
            end: Local.with_ymd_and_hms(2022, 1, 10, 10, 10, 0).unwrap(),
        };
        failed.label = Some("label-10-0".to_string());
        tasks.insert(failed.id, failed);

        // Successful task
        let mut successful = build_task();
        successful.id = 1;
        successful.status = TaskStatus::Done {
            result: TaskResult::Success,
            enqueued_at: Local.with_ymd_and_hms(2022, 1, 8, 10, 0, 0).unwrap(),
            start: Local.with_ymd_and_hms(2022, 1, 8, 10, 5, 0).unwrap(),
            end: Local.with_ymd_and_hms(2022, 1, 8, 10, 10, 0).unwrap(),
        };
        successful.label = Some("label-10-1".to_string());
        tasks.insert(successful.id, successful);

        // Stashed task
        let mut stashed = build_task();
        stashed.status = TaskStatus::Stashed { enqueue_at: None };
        stashed.id = 2;
        stashed.label = Some("label-10-2".to_string());
        tasks.insert(stashed.id, stashed);

        // Scheduled task
        let mut scheduled = build_task();
        scheduled.status = TaskStatus::Stashed {
            enqueue_at: Some(Local.with_ymd_and_hms(2022, 1, 10, 11, 0, 0).unwrap()),
        };
        scheduled.id = 3;
        scheduled.group = "testgroup".to_string();
        tasks.insert(scheduled.id, scheduled);

        // Running task
        let mut running = build_task();
        running.status = TaskStatus::Running {
            enqueued_at: Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap(),
            start: Local.with_ymd_and_hms(2022, 1, 2, 12, 0, 0).unwrap(),
        };
        running.id = 4;
        tasks.insert(running.id, running);

        // Add two queued tasks with different command
        let mut queued = build_task();
        queued.id = 5;
        queued.command = TEST_COMMAND_HELLO.to_string();
        tasks.insert(queued.id, queued.clone());

        // Task 6 depends on task 5
        queued.id = 6;
        queued.dependencies.push(5);
        tasks.insert(queued.id, queued);

        tasks
    }

    fn test_tasks_with_query(query: &str, group: &Option<String>) -> Result<Vec<Task>> {
        let mut tasks = test_tasks();

        let query_result = apply_query(query, group)?;
        tasks = query_result.apply_filters(tasks);
        tasks = query_result.order_tasks(tasks);
        tasks = query_result.limit_tasks(tasks);

        Ok(tasks)
    }

    /// Select only specific columns for printing
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn column_selection() -> Result<()> {
        let result = apply_query("columns=id,status,command", &None)?;
        assert_eq!(
            result.selected_columns,
            [Rule::column_id, Rule::column_status, Rule::column_command]
        );

        Ok(())
    }

    /// Select the first few entries of the list
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn limit_first() -> Result<()> {
        let tasks = test_tasks_with_query("first 4", &None)?;

        assert!(tasks.len() == 4);
        assert_eq!(tasks[0].id, 0);
        assert_eq!(tasks[3].id, 3);

        Ok(())
    }

    /// Select the last few entries of the list
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn limit_last() -> Result<()> {
        let tasks = test_tasks_with_query("last 4", &None)?;

        assert!(tasks.len() == 4);
        assert_eq!(tasks[0].id, 3);
        assert_eq!(tasks[3].id, 6);

        Ok(())
    }

    /// Filter by start date
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn filter_start() -> Result<()> {
        let tasks = test_tasks_with_query("start>2022-01-10 09:00:00", &None)?;

        assert!(tasks.len() == 1);
        assert_eq!(tasks[0].id, 0);

        Ok(())
    }

    /// Filtering in combination with groups works as expected
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn filter_with_group() -> Result<()> {
        let tasks = test_tasks_with_query("status=stashed", &Some("testgroup".to_string()))?;

        assert!(tasks.len() == 1);
        assert_eq!(tasks[0].id, 3);

        Ok(())
    }

    /// Filter by end date with the current time as a time and a date.
    #[rstest]
    #[case("2022-01-10")]
    #[case("2022-01-10 09:00:00")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn filter_end_with_time(#[case] format: &'static str) -> Result<()> {
        let tasks = test_tasks_with_query(&format!("end<{format}"), &None)?;

        assert!(tasks.len() == 1);
        assert_eq!(tasks[0].id, 1);

        Ok(())
    }

    /// Filter tasks by status
    #[rstest]
    #[case("queued", 2)]
    #[case("running", 1)]
    #[case("paused", 0)]
    #[case("success", 1)]
    #[case("failed", 1)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn filter_status(#[case] status_filter: &str, #[case] match_count: usize) -> Result<()> {
        // Get the correct query keyword for the given status.

        let tasks = test_tasks_with_query(&format!("status={status_filter}"), &None)?;

        for task in tasks.iter() {
            match status_filter {
                "queued" => {
                    assert_matches!(
                        task.status,
                        TaskStatus::Queued { .. },
                        "Only Queued tasks are allowed"
                    );
                }
                "stashed" => assert_matches!(
                    task.status,
                    TaskStatus::Stashed { .. },
                    "Only Stashed tasks are allowed"
                ),
                "running" => assert_matches!(
                    task.status,
                    TaskStatus::Running { .. },
                    "Only Running tasks are allowed"
                ),
                "paused" => assert_matches!(
                    task.status,
                    TaskStatus::Paused { .. },
                    "Only Paused tasks are allowed"
                ),
                "success" => assert_matches!(
                    task.status,
                    TaskStatus::Done {
                        result: TaskResult::Success,
                        ..
                    },
                    "Only Succesful tasks are allowed"
                ),
                "failed" => assert_matches!(
                    task.status,
                    TaskStatus::Done {
                        result: TaskResult::Failed(_),
                        ..
                    },
                    "Only Failed tasks are allowed"
                ),
                _ => bail!("Got unexpected TaskStatus in filter_status"),
            };
        }

        assert_eq!(
        tasks.len(),
        match_count,
        "Got a different amount of tasks than expected for the status filter {status_filter:?}."
    );

        Ok(())
    }

    /// Order the test state by task status.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn order_by_status() -> Result<()> {
        let tasks = test_tasks_with_query("order_by status", &None)?;

        let expected = vec![
            TaskStatus::Stashed { enqueue_at: None },
            TaskStatus::Stashed {
                enqueue_at: Some(Local.with_ymd_and_hms(2022, 1, 10, 11, 0, 0).unwrap()),
            },
            TaskStatus::Queued {
                enqueued_at: Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap(),
            },
            TaskStatus::Queued {
                enqueued_at: Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap(),
            },
            TaskStatus::Running {
                enqueued_at: Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap(),
                start: Local.with_ymd_and_hms(2022, 1, 2, 12, 0, 0).unwrap(),
            },
            TaskStatus::Done {
                enqueued_at: Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap(),
                start: Local.with_ymd_and_hms(2022, 1, 10, 10, 5, 0).unwrap(),
                end: Local.with_ymd_and_hms(2022, 1, 10, 10, 10, 0).unwrap(),
                result: TaskResult::Failed(255),
            },
            TaskStatus::Done {
                enqueued_at: Local.with_ymd_and_hms(2022, 1, 8, 10, 0, 0).unwrap(),
                start: Local.with_ymd_and_hms(2022, 1, 8, 10, 5, 0).unwrap(),
                end: Local.with_ymd_and_hms(2022, 1, 8, 10, 10, 0).unwrap(),
                result: TaskResult::Success,
            },
        ];

        let actual: Vec<TaskStatus> = tasks.iter().map(|task| task.status.clone()).collect();
        assert_eq!(actual, expected);

        Ok(())
    }

    /// Order the tasks by enqueue(d) date.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn order_by_enqueue_at() -> Result<()> {
        let tasks = test_tasks_with_query("order_by enqueue_at asc", &None)?;

        let expected = vec![
            TaskStatus::Done {
                enqueued_at: Local.with_ymd_and_hms(2022, 1, 8, 10, 0, 0).unwrap(),
                start: Local.with_ymd_and_hms(2022, 1, 8, 10, 5, 0).unwrap(),
                end: Local.with_ymd_and_hms(2022, 1, 8, 10, 10, 0).unwrap(),
                result: TaskResult::Success,
            },
            TaskStatus::Done {
                enqueued_at: Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap(),
                start: Local.with_ymd_and_hms(2022, 1, 10, 10, 5, 0).unwrap(),
                end: Local.with_ymd_and_hms(2022, 1, 10, 10, 10, 0).unwrap(),
                result: TaskResult::Failed(255),
            },
            TaskStatus::Running {
                enqueued_at: Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap(),
                start: Local.with_ymd_and_hms(2022, 1, 2, 12, 0, 0).unwrap(),
            },
            TaskStatus::Queued {
                enqueued_at: Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap(),
            },
            TaskStatus::Queued {
                enqueued_at: Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap(),
            },
            TaskStatus::Stashed {
                enqueue_at: Some(Local.with_ymd_and_hms(2022, 1, 10, 11, 0, 0).unwrap()),
            },
            TaskStatus::Stashed { enqueue_at: None },
        ];

        let actual: Vec<TaskStatus> = tasks.iter().map(|task| task.status.clone()).collect();
        assert_eq!(actual, expected);

        Ok(())
    }

    /// Filter tasks by label with the "eq" `=` "ne" `!=` and "contains" `%=`filter.
    #[rstest]
    #[case("%=", "label", 3)]
    #[case("%=", "label-10", 3)]
    #[case("%=", "label-10-1", 1)]
    #[case("=", "label-10-1", 1)]
    #[case("!=", "label-10-1", 6)]
    #[case("!=", "label-10", 7)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn filter_label(
        #[case] operator: &'static str,
        #[case] label_filter: &'static str,
        #[case] match_count: usize,
    ) -> Result<()> {
        let tasks = test_tasks_with_query(&format!("label{operator}{label_filter}"), &None)?;

        for task in tasks.iter() {
            // Make sure the task either has no label or the label doesn't match the filter.
            if operator == "!=" {
                if let Some(label) = &task.label {
                    assert_ne!(
                        label, label_filter,
                        "Label '{label}' matched exact filter '{label_filter}'"
                    );
                }
                continue;
            }

            let label = task.label.as_ref().expect("Expected task to have a label");
            if operator == "%=" {
                // Make sure the label contained our filter.
                assert!(
                    label.contains(label_filter),
                    "Label '{label}' didn't contain filter '{label_filter}'"
                );
            } else if operator == "=" {
                // Make sure the label exactly matches the filter.
                assert_eq!(
                    label, &label_filter,
                    "Label '{label}' didn't match exact filter '{label_filter}'"
                );
            }
        }

        assert_eq!(
            tasks.len(),
            match_count,
            "Got a different amount of tasks than expected for the label filter: {label_filter}."
        );

        Ok(())
    }

    /// Filter tasks by command with the "eq" `=` "ne" `!=` and "contains" `%=`filter.
    #[rstest]
    #[case("=", TEST_COMMAND_SLEEP, 5)]
    #[case("!=", TEST_COMMAND_SLEEP, 2)]
    #[case("%=", &TEST_COMMAND_SLEEP[..4], 5)]
    #[case("=", TEST_COMMAND_HELLO, 2)]
    #[case("!=", TEST_COMMAND_HELLO, 5)]
    #[case("%=", &TEST_COMMAND_HELLO[..4], 2)]
    #[case("!=", "nonexist", 7)]
    #[case("%=", "nonexist", 0)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn filter_command(
        #[case] operator: &'static str,
        #[case] command_filter: &'static str,
        #[case] match_count: usize,
    ) -> Result<()> {
        let tasks = test_tasks_with_query(&format!("command{operator}{command_filter}"), &None)?;

        for task in tasks.iter() {
            let command = task.command.as_str();
            if operator == "!=" {
                // Make sure the task's command doesn't match the filter.
                assert_ne!(
                    command, command_filter,
                    "Command '{command}' matched exact filter '{command_filter}'"
                );
            } else if operator == "%=" {
                // Make sure the command contained our filter.
                assert!(
                    command.contains(command_filter),
                    "Command '{command}' didn't contain filter '{command_filter}'"
                );
            } else if operator == "=" {
                // Make sure the command exactly matches the filter.
                assert_eq!(
                    command, command_filter,
                    "Command '{command}' didn't match exact filter '{command_filter}'"
                );
            }
        }

        assert_eq!(
        tasks.len(),
        match_count,
        "Got a different amount of tasks than expected for the command filter: {command_filter}."
    );

        Ok(())
    }
}
