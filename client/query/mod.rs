use anyhow::{bail, Context, Result};
use pest::Parser;
use pest_derive::Parser;

use crate::display::table_builder::TableBuilder;

mod column_selection;

#[derive(Parser)]
#[grammar = "./client/query/syntax.pest"]
struct QueryParser;

/// Take a given `pueue status QUERY` and apply it to all components that're involved in the
/// `pueue status` process:
///
/// - TableBuilder: The component responsible for building the table and determining which
///         columns should or need to be displayed.
///         A `columns [columns]` statement will define the set of visible columns.
pub fn apply_query(query: String, table_builder: &mut TableBuilder) -> Result<()> {
    let mut parsed = QueryParser::parse(Rule::query, &query).context("Failed to parse query")?;
    dbg!(&parsed);

    // Expect there to be exactly one pair for the full query.
    // Return early if we got an empty query.
    let parsed = if let Some(pair) = parsed.next() {
        pair
    } else {
        return Ok(());
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
        if section.as_rule() == Rule::column_selection {
            column_selection::apply(section, table_builder)?;
        }
    }

    Ok(())
}
