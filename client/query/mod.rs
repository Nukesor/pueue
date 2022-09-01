use anyhow::{bail, Context, Result};
use pest::Parser;
use pest_derive::Parser;

use pueue_lib::task::Task;

mod column_selection;
mod filters;

#[derive(Parser)]
#[grammar = "./client/query/syntax.pest"]
struct QueryParser;

/// All appliable information that has been extracted from the query.
#[derive(Default)]
pub struct QueryResult {
    /// The list of selected columns based.
    pub selected_columns: Vec<Rule>,

    /// A list of filter functions that should be applied to the list of tasks.
    filters: Vec<Box<dyn Fn(&Task) -> bool>>,
}

impl QueryResult {
    /// Take a list of tasks and apply all filters to it.
    pub fn apply_filters(&self, tasks: Vec<Task>) -> Vec<Task> {
        let mut iter = tasks.into_iter();
        for filter in self.filters.iter() {
            iter = iter.filter(filter).collect::<Vec<Task>>().into_iter();
        }
        iter.collect()
    }
}

/// Take a given `pueue status QUERY` and apply it to all components that're involved in the
/// `pueue status` process:
///
/// - TableBuilder: The component responsible for building the table and determining which
///         columns should or need to be displayed.
///         A `columns [columns]` statement will define the set of visible columns.
pub fn apply_query(query: String) -> Result<QueryResult> {
    let mut parsed = QueryParser::parse(Rule::query, &query).context("Failed to parse query")?;

    let mut query_result = QueryResult::default();

    // Expect there to be exactly one pair for the full query.
    // Return early if we got an empty query.
    let parsed = if let Some(pair) = parsed.next() {
        pair
    } else {
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
            _ => (),
        }
    }

    Ok(query_result)
}
