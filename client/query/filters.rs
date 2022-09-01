use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use pest::iterators::Pair;
use pueue_lib::task::{Task, TaskStatus};

use super::{QueryResult, Rule};

enum DateOrDateTime {
    DateTime(DateTime<Local>),
    Date(NaiveDate),
}

/// Parse a datetime/date/time filter.
/// Such a filter can be applied to either the `start`, `end` or `enqueue_at` field.
///
/// This filter syntax looks like this is expected to be:
/// `[enqueue_at|start|end] [>|<|=|!=] [YYYY-MM-DD HH:mm:SS|HH:mm:SS|YYYY-MM-DD]`
///
/// The datastructer looks something like this:
/// Pair {
///     rule: datetime_filter,
///     span: Span {
///         str: "start=2022-09-01",
///         start: 0,
///         end: 16,
///     },
///     inner: [
///         Pair {
///             rule: column_start,
///             span: Span {
///                 str: "start",
///                 start: 0,
///                 end: 5,
///             },
///             inner: [],
///         },
///         Pair {
///             rule: operator,
///             span: Span {
///                 str: "=",
///                 start: 5,
///                 end: 6,
///             },
///             inner: [
///                 Pair {
///                     rule: eq,
///                     span: Span {
///                         str: "=",
///                         start: 5,
///                         end: 6,
///                     },
///                     inner: [],
///                 },
///             ],
///         },
///         Pair {
///             rule: date,
///             span: Span {
///                 str: "2022-09-01",
///                 start: 6,
///                 end: 16,
///             },
///             inner: [],
///         },
///     ],
/// }
pub fn datetime<'i>(section: Pair<'i, Rule>, query_result: &mut QueryResult) -> Result<()> {
    let mut filter = section.into_inner();
    // Get the column this filter should be applied to.
    let column = filter.next().unwrap();
    let column = column.as_rule();
    match column {
        Rule::column_enqueue_at | Rule::column_start | Rule::column_end => (),
        _ => bail!("Expected either of [enqueue_at,start,stop]"),
    }

    // Get the operator that should be applied in this filter.
    let operator = filter.next().unwrap().as_rule();
    match operator {
        Rule::eq | Rule::neq | Rule::lt | Rule::gt => (),
        _ => bail!("Expected a comparison operator for date/time filters"),
    }

    // Get the
    let operand = filter.next().unwrap();
    let operand_rule = operand.as_rule();
    let operand = match operand_rule {
        Rule::time => {
            let time = NaiveTime::parse_from_str(operand.as_str(), "%X")
                .context("Expected hh:mm:ss time format")?;
            let date = Local::today();
            DateOrDateTime::DateTime(date.and_time(time).unwrap())
        }
        Rule::datetime => {
            let datetime = NaiveDateTime::parse_from_str(operand.as_str(), "%F %X")
                .context("Expected YYYY-MM-SS hh:mm:ss date time format")?;
            DateOrDateTime::DateTime(datetime.and_local_timezone(Local).unwrap())
        }
        Rule::date => {
            let date = NaiveDate::parse_from_str(operand.as_str(), "%F")
                .context("Expected YYYY-MM-SS date format")?;
            DateOrDateTime::Date(date)
        }
        _ => bail!("Expected either a date, datetime or time expression."),
    };

    let filter_function = Box::new(move |task: &Task| -> bool {
        let field = match column {
            Rule::column_enqueue_at => {
                if let TaskStatus::Stashed {
                    enqueue_at: Some(enqueue_at),
                } = task.status
                {
                    enqueue_at
                } else {
                    return false;
                }
            }
            Rule::column_start => {
                if let Some(start) = task.start {
                    start
                } else {
                    return false;
                }
            }
            Rule::column_end => {
                if let Some(end) = task.end {
                    end
                } else {
                    return false;
                }
            }
            _ => return true,
        };
        // Apply the operator to the operands.
        // The operator might have a different meaning depending on the type of datetime/date
        // we're dealing with.
        // E.g. when working with dates, `>` should mean bigger than the end of that day.
        // `<` however should mean before that day.
        match operand {
            DateOrDateTime::DateTime(datetime) => match operator {
                Rule::eq => field == datetime,
                Rule::neq => field != datetime,
                Rule::lt => field < datetime,
                Rule::gt => field > datetime,
                _ => true,
            },
            DateOrDateTime::Date(date) => {
                // Get the start of the given day.
                // Use the most inclusive datetime in case of ambiguity
                let start_of_day = date.and_hms(0, 0, 0).and_local_timezone(Local);
                let start_of_day = match start_of_day.latest() {
                    None => return false,
                    Some(datetime) => datetime,
                };

                // Get the end of the given day.
                // Use the most inclusive datetime in case of ambiguity
                let end_of_day = (date + Duration::days(1))
                    .and_hms(0, 0, 0)
                    .and_local_timezone(Local);
                let end_of_day = match end_of_day.latest() {
                    None => return false,
                    Some(datetime) => datetime,
                };

                match operator {
                    Rule::eq => field > start_of_day && field < end_of_day,
                    Rule::neq => field < start_of_day && field > end_of_day,
                    Rule::lt => field < start_of_day,
                    Rule::gt => field > end_of_day,
                    _ => true,
                }
            }
        }
    });
    query_result.filters.push(filter_function);

    Ok(())
}

/// Parse a filter for the label fiel.
///
/// This filter syntax looks like this:
/// `label [=|!=] string`
///
/// The datastructer looks something like this:
///  Pair {
///     rule: label_filter,
///     span: Span {
///         str: "label=test",
///         start: 0,
///         end: 10,
///     },
///     inner: [
///         Pair {
///             rule: column_label,
///             span: Span {
///                 str: "label",
///                 start: 0,
///                 end: 5,
///             },
///             inner: [],
///         },
///         Pair {
///             rule: eq,
///             span: Span {
///                 str: "=",
///                 start: 5,
///                 end: 6,
///             },
///             inner: [],
///         },
///         Pair {
///             rule: label,
///             span: Span {
///                 str: "test",
///                 start: 6,
///                 end: 10,
///             },
///             inner: [],
///         },
///     ],
/// }
pub fn label<'i>(section: Pair<'i, Rule>, query_result: &mut QueryResult) -> Result<()> {
    dbg!(&section);
    let mut filter = section.into_inner();
    // The first word should be the `label` keyword.
    let column = filter.next().unwrap();
    match column.as_rule() {
        Rule::column_label => (),
        _ => bail!("Expected label keyword"),
    }

    // Get the operator that should be applied in this filter.
    let operator = filter.next().unwrap().as_rule();
    match operator {
        Rule::eq | Rule::neq => (),
        _ => bail!("Expected a [=|!=] comparison operator label filter"),
    }

    // Get the name of the label we should filter for.
    let operand = filter.next().unwrap().as_str().to_string();

    // Filter for the label
    let filter_function = Box::new(move |task: &Task| -> bool {
        let label = if let Some(label) = &task.label {
            label
        } else {
            return operator == Rule::neq;
        };

        match operator {
            Rule::eq => label == &operand,
            Rule::neq => label != &operand,
            _ => false,
        }
    });
    query_result.filters.push(filter_function);

    Ok(())
}
