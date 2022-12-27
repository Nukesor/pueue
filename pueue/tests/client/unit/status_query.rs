use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use chrono::{Local, TimeZone};
use pretty_assertions::assert_eq;
use rstest::rstest;

use pueue::client::query::{apply_query, Rule};
use pueue_lib::state::PUEUE_DEFAULT_GROUP;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

/// A small helper function to reduce a bit of boilerplate.
pub fn build_task() -> Task {
    Task::new(
        "sleep 60".to_owned(),
        PathBuf::from("/tmp"),
        HashMap::new(),
        PUEUE_DEFAULT_GROUP.to_owned(),
        TaskStatus::Queued,
        Vec::new(),
        None,
    )
}

/// Build a list of some pre-build tasks that are used to test.
pub fn test_tasks() -> Vec<Task> {
    let mut tasks = Vec::new();

    // Failed task
    let mut failed = build_task();
    failed.id = 0;
    failed.status = TaskStatus::Done(TaskResult::Failed(255));
    failed.start = Some(Local.with_ymd_and_hms(2022, 1, 10, 10, 0, 0).unwrap());
    failed.end = Some(Local.with_ymd_and_hms(2022, 1, 10, 10, 5, 0).unwrap());
    failed.label = Some("label-10-0".to_string());
    tasks.insert(failed.id, failed);

    // Successful task
    let mut successful = build_task();
    successful.id = 1;
    successful.status = TaskStatus::Done(TaskResult::Success);
    successful.start = Some(Local.with_ymd_and_hms(2022, 1, 8, 10, 0, 0).unwrap());
    successful.end = Some(Local.with_ymd_and_hms(2022, 1, 8, 10, 5, 0).unwrap());
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
    tasks.insert(scheduled.id, scheduled);

    // Running task
    let mut running = build_task();
    running.status = TaskStatus::Running;
    running.id = 4;
    running.start = Some(Local.with_ymd_and_hms(2022, 1, 2, 12, 0, 0).unwrap());
    tasks.insert(running.id, running);

    // Add two queued tasks
    let mut queued = build_task();
    queued.id = 5;
    tasks.insert(queued.id, queued.clone());

    // Task 6 depends on task 5
    queued.id = 6;
    queued.dependencies.push(5);
    tasks.insert(queued.id, queued);

    tasks
}

fn test_tasks_with_query(query: &str) -> Result<Vec<Task>> {
    let mut tasks = test_tasks();

    let query_result = apply_query(query)?;
    tasks = query_result.apply_filters(tasks);
    tasks = query_result.order_tasks(tasks);
    tasks = query_result.limit_tasks(tasks);

    Ok(tasks)
}

/// Select only specific columns for printing
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn column_selection() -> Result<()> {
    let result = apply_query("columns=id,status,command")?;
    assert_eq!(
        result.selected_columns,
        [Rule::column_id, Rule::column_status, Rule::column_command]
    );

    Ok(())
}

/// Select the first few entries of the list
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn limit_first() -> Result<()> {
    let tasks = test_tasks_with_query("first 4")?;

    assert!(tasks.len() == 4);
    assert_eq!(tasks[0].id, 0);
    assert_eq!(tasks[3].id, 3);

    Ok(())
}

/// Select the last few entries of the list
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn limit_last() -> Result<()> {
    let tasks = test_tasks_with_query("last 4")?;

    assert!(tasks.len() == 4);
    assert_eq!(tasks[0].id, 3);
    assert_eq!(tasks[3].id, 6);

    Ok(())
}

/// Order the test state by task status.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn order_by_status() -> Result<()> {
    let tasks = test_tasks_with_query("order_by status")?;

    let expected = vec![
        TaskStatus::Stashed { enqueue_at: None },
        TaskStatus::Stashed {
            enqueue_at: Some(Local.with_ymd_and_hms(2022, 1, 10, 11, 0, 0).unwrap()),
        },
        TaskStatus::Queued,
        TaskStatus::Queued,
        TaskStatus::Running,
        TaskStatus::Done(TaskResult::Failed(255)),
        TaskStatus::Done(TaskResult::Success),
    ];

    let actual: Vec<TaskStatus> = tasks.iter().map(|task| task.status.clone()).collect();
    assert_eq!(actual, expected);

    Ok(())
}

/// Filter by start date
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn filter_start() -> Result<()> {
    let tasks = test_tasks_with_query("start>2022-01-10 09:00:00")?;

    assert!(tasks.len() == 1);
    assert_eq!(tasks[0].id, 0);

    Ok(())
}

/// Filter by end date with the current time as a time and a date.
#[rstest]
#[case("2022-01-10")]
#[case("2022-01-10 09:00:00")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn filter_end_with_time(#[case] format: &'static str) -> Result<()> {
    let tasks = test_tasks_with_query(&format!("end<{format}"))?;

    assert!(tasks.len() == 1);
    assert_eq!(tasks[0].id, 1);

    Ok(())
}

/// Filter tasks by status
#[rstest]
#[case(TaskStatus::Queued, 2)]
#[case(TaskStatus::Running, 1)]
#[case(TaskStatus::Paused, 0)]
#[case(TaskStatus::Done(TaskResult::Success), 1)]
#[case(TaskStatus::Done(TaskResult::Failed(255)), 1)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn filter_status(#[case] status: TaskStatus, #[case] match_count: usize) -> Result<()> {
    // Get the correct query keyword for the given status.
    let status_filter = match status {
        TaskStatus::Queued => "queued",
        TaskStatus::Stashed { .. } => "stashed",
        TaskStatus::Running => "running",
        TaskStatus::Paused => "paused",
        TaskStatus::Done(TaskResult::Success) => "success",
        TaskStatus::Done(TaskResult::Failed(_)) => "failed",
        _ => anyhow::bail!("Got unexpected TaskStatus in filter_status"),
    };

    let tasks = test_tasks_with_query(&format!("status={status_filter}"))?;

    for task in tasks.iter() {
        let id = task.id;
        assert_eq!(
            task.status, status,
            "Expected a different task status on task {id} based on filter {status:?}"
        );
    }

    assert_eq!(
        tasks.len(),
        match_count,
        "Got a different amount of tasks than expected for the status filter {status:?}."
    );

    Ok(())
}

/// Filter tasks by label with the "contains" `%=` filter.
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
    let tasks = test_tasks_with_query(&format!("label{operator}{label_filter}"))?;

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
