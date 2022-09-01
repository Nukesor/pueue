use anyhow::{bail, ensure, Context, Result};
use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;

use crate::display::table_builder::TableBuilder;

#[derive(Parser)]
#[grammar = "./client/query.pest"]
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
            // This query is expected to be the "columns" keyword + columns
            let mut columns_pairs = section.into_inner();
            let columns_word = columns_pairs
                .next()
                .context("Expected 'columns' keyword in column selection")?;
            ensure!(
                columns_word.as_rule() == Rule::columns_word,
                "Expected leading 'columns' keyword in columns query"
            );

            let equals = columns_pairs
                .next()
                .context("Expected columns after 'columns' in column selection")?;
            ensure!(
                equals.as_rule() == Rule::eq,
                "Expected multiple columns after 'columns' keyword in column selection"
            );

            let columns = columns_pairs
                .next()
                .context("Expected columns after 'columns' in column selection")?;
            ensure!(
                columns.as_rule() == Rule::multiple_columns,
                "Expected multiple columns after 'columns' keyword in column selection"
            );
            apply_column_selection(columns.into_inner(), table_builder)?;
        }
    }

    Ok(())
}

fn apply_column_selection<'i>(
    columns: Pairs<'i, Rule>,
    table_builder: &mut TableBuilder,
) -> Result<()> {
    // Extract all columns from the multiple_columns.inner iterator
    // The structure is like this
    // ```
    // Pair {
    //  rule: multiple_columns,
    //  span: Span {
    //      str: "id,status",
    //      start: 7,
    //      end: 16,
    //  },
    //  inner: [
    //      Pair {
    //          rule: column,
    //          span: Span {
    //              str: "id",
    //              start: 7,
    //              end: 9,
    //          },
    //          inner: [
    //              Pair {
    //                  rule: id,
    //                  span: Span {
    //                      str: "id",
    //                      start: 7,
    //                      end: 9,
    //                  },
    //                  inner: [],
    //              },
    //          ],
    //      },
    //      ...
    //      ]
    // }
    // ```
    let columns = columns
        .map(|pair| {
            pair.into_inner()
                .next()
                .context("Expected a column in the column selection.")
                .map(|inner_pair| inner_pair.as_rule())
        })
        .collect::<Result<Vec<Rule>>>()?;

    table_builder.set_visibility_by_rules(columns);

    Ok(())
}
