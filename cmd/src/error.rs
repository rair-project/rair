//! error.rs: rcmd Error handling mechanism.

use crate::grammar::Rule;
use err_derive::Error;
use pest::error;
use pest::iterators::Pair;
use std::num;

#[derive(Debug, Error, PartialEq)]
pub enum ParserError {
    #[error(display = "{})", _0)]
    Num(num::ParseIntError),
    #[error(display = "{})", _0)]
    Pest(Box<error::Error<Rule>>),
}

pub fn unimplemented_pair(root: Pair<Rule>) -> ! {
    println!("{:#?}", root);
    unimplemented!();
}
