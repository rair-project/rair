//! error.rs: rcmd Error handling mechanism.

use crate::grammar::Rule;
use pest::error;
use pest::iterators::Pair;
use std::num;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParserError {
    #[error("{0}")]
    Num(#[source] num::ParseIntError),
    #[error("{0}")]
    Pest(#[source] Box<error::Error<Rule>>),
}

#[expect(
    clippy::print_stdout,
    clippy::use_debug,
    clippy::unimplemented,
    reason = "diagnostic dump for grammar rules that the pest grammar makes unreachable"
)]
pub fn unimplemented_pair(root: &Pair<Rule>) -> ! {
    println!("{root:#?}");
    unimplemented!();
}
