//! Generate AST like structure for rair commands.

use crate::{
    cmd::Cmd,
    error::{unimplemented_pair, ParserError},
    grammar::{CliParser, Rule},
    help::HelpCmd,
};
use pest::Parser;

#[derive(Debug, PartialEq)]
pub enum ParseTree {
    Help(HelpCmd),
    Cmd(Cmd),
    Comment,
    NewLine,
    HelpAll,
}

impl ParseTree {
    pub fn construct(line: &str) -> Result<Self, ParserError> {
        let pairs = CliParser::parse(Rule::Input, line);
        if pairs.is_err() {
            return Err(ParserError::Pest(Box::new(pairs.err().unwrap())));
        }
        let pair = pairs.unwrap().next().unwrap().into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::HelpLine => Ok(Self::Help(HelpCmd::parse_help(pair))),
            Rule::Comment => Ok(Self::Comment),
            Rule::EmptyLine => Ok(Self::NewLine),
            Rule::CommandLine => Ok(Self::Cmd(Cmd::parse_cmd(pair)?)),
            Rule::HelpAll => Ok(ParseTree::HelpAll),
            Rule::EOI
            | Rule::WHITESPACE
            | Rule::CustomAlpha
            | Rule::CustomAlphaNum
            | Rule::ANS
            | Rule::ANWS
            | Rule::DEC
            | Rule::BIN
            | Rule::HEX
            | Rule::OCT
            | Rule::Command
            | Rule::ArgumentLiteral
            | Rule::Argument
            | Rule::Arguments
            | Rule::Loc
            | Rule::Pipe
            | Rule::Red
            | Rule::RedCat
            | Rule::RedPipe
            | Rule::Input => unimplemented_pair(&pair),
        }
    }
}

#[cfg(test)]
mod test_parser {
    use super::*;
    #[test]
    fn test_parser() {
        let mut tree = ParseTree::construct("aa? #and a little comment").unwrap();
        assert_eq!(
            tree,
            ParseTree::Help(HelpCmd {
                command: "aa".to_owned()
            })
        );
        ParseTree::construct("aa withargument? #and a little comment").unwrap_err();
        tree = ParseTree::construct("aa #and a little comment").unwrap();
        let cmd: Cmd = Cmd {
            command: "aa".to_owned(),
            ..Default::default()
        };
        assert_eq!(tree, ParseTree::Cmd(cmd));
        tree = ParseTree::construct("#and a little comment").unwrap();
        assert_eq!(tree, ParseTree::Comment);
        tree = ParseTree::construct("").unwrap();
        assert_eq!(tree, ParseTree::NewLine);
    }
}
