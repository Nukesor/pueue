use anyhow::{ensure, Context, Result};
use pest::iterators::Pair;

use super::{QueryResult, Rule};

pub fn apply<'i>(section: Pair<'i, Rule>, query_result: &mut QueryResult) -> Result<()> {
    Ok(())
}
