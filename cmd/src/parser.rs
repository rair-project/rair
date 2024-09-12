//! Generate AST like structure for rair commands.

use crate::{cmd::Cmd, error::*, grammar::*, help::HelpCmd};
use pest::Parser;

#[derive(Debug, PartialEq)]
pub enum ParseTree {
    Help(HelpCmd),
    Cmd(Cmd),
    Comment,
    NewLine,
}

impl ParseTree {
    pub fn construct(line: &str) -> Result<Self, ParserError> {
        let pairs = CliParser::parse(Rule::Input, line);
        if pairs.is_err() {
            return Err(ParserError::Pest(Box::new(pairs.err().unwrap())));
        }
        let pair = pairs.unwrap().next().unwrap().into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::HelpLine => Ok(ParseTree::Help(HelpCmd::parse_help(pair))),
            Rule::Comment => Ok(ParseTree::Comment),
            Rule::EmptyLine => Ok(ParseTree::NewLine),
            Rule::CommandLine => Ok(ParseTree::Cmd(Cmd::parse_cmd(pair)?)),
            _ => unimplemented_pair(pair),
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
                command: "aa".to_string()
            })
        );
        assert!(ParseTree::construct("aa withargument? #and a little comment").is_err());
        tree = ParseTree::construct("aa #and a little comment").unwrap();
        let mut cmd: Cmd = Default::default();
        cmd.command = "aa".to_string();
        assert_eq!(tree, ParseTree::Cmd(cmd));
        tree = ParseTree::construct("#and a little comment").unwrap();
        assert_eq!(tree, ParseTree::Comment);
        tree = ParseTree::construct("").unwrap();
        assert_eq!(tree, ParseTree::NewLine);
    }
}
