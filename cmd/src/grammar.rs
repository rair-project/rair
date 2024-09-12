//! Deriving grammar for rair from cli.pest.

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "cli.pest"]
pub(crate) struct CliParser;
