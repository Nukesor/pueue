use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./client/query.pest"]
pub struct QueryParser;
