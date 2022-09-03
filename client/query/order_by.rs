use anyhow::{bail, ensure, Result};
use pest::iterators::Pair;

use super::{Direction, QueryResult, Rule};

/// Parse an order_by condition.
///
/// This filter syntax looks like this:
/// `order_by [column] [asc|desc]`
///
/// The data structure looks something like this:
/// Pair {
///     rule: order_by_condition,
///     span: Span {
///         str: "order_by label desc",
///         start: 0,
///         end: 19,
///     },
///     inner: [
///         Pair {
///             rule: order_by,
///             span: Span {
///                 str: "order_by",
///                 start: 0,
///                 end: 8,
///             },
///             inner: [],
///         },
///         Pair {
///             rule: column,
///             span: Span {
///                 str: "label",
///                 start: 9,
///                 end: 14,
///             },
///             inner: [
///                 Pair {
///                     rule: column_label,
///                     span: Span {
///                         str: "label",
///                         start: 9,
///                         end: 14,
///                     },
///                     inner: [],
///                 },
///             ],
///         },
///         Pair {
///             rule: descending,
///             span: Span {
///                 str: "desc",
///                 start: 15,
///                 end: 19,
///             },
///             inner: [],
///         },
///     ],
/// }
pub fn order_by(section: Pair<'_, Rule>, query_result: &mut QueryResult) -> Result<()> {
    let mut order_by_condition = section.into_inner();
    // The first word should be the `label` keyword.
    let order_by = order_by_condition.next().unwrap();
    match order_by.as_rule() {
        Rule::order_by => (),
        _ => bail!("Expected order_by keyword"),
    }

    // Get the label we should order by.
    let column_keyword = order_by_condition.next().unwrap();
    ensure!(
        column_keyword.as_rule() == Rule::column,
        "Expected multiple columns after 'columns' keyword in column selection"
    );
    let column = column_keyword.into_inner().next().unwrap().as_rule();

    // Get the name of the label we should filter for.
    let direction = match order_by_condition.next().unwrap().as_rule() {
        Rule::ascending => Direction::Ascending,
        Rule::descending => Direction::Descending,
        _ => return Ok(()),
    };

    query_result.order_by = Some((column, direction));
    Ok(())
}
