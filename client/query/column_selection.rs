use anyhow::{ensure, Context, Result};
use pest::iterators::Pair;

use super::{QueryResult, Rule};

pub fn apply(section: Pair<'_, Rule>, query_result: &mut QueryResult) -> Result<()> {
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

    let multiple_columns = columns_pairs
        .next()
        .context("Expected columns after 'columns' in column selection")?;
    ensure!(
        multiple_columns.as_rule() == Rule::multiple_columns,
        "Expected multiple columns after 'columns' keyword in column selection"
    );

    let columns = multiple_columns.into_inner();
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
    let mut columns = columns
        .map(|pair| {
            pair.into_inner()
                .next()
                .context("Expected a column in the column selection.")
                .map(|inner_pair| inner_pair.as_rule())
        })
        .collect::<Result<Vec<Rule>>>()?;

    query_result.selected_columns.append(&mut columns);

    Ok(())
}
