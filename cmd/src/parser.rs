/*
 * parser.rs: Generate AST like structure for rair commands.
 * Copyright (C) 2019  Oddcoder
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use cmd::Cmd;
use error::*;
use grammar::*;
use help::HelpCmd;
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
            return Err(ParserError::Pest(pairs.err().unwrap()));
        }
        let pair = pairs.unwrap().next().unwrap().into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::HelpLine => return Ok(ParseTree::Help(HelpCmd::parse_help(pair))),
            Rule::Comment => return Ok(ParseTree::Comment),
            Rule::EmptyLine => return Ok(ParseTree::NewLine),
            Rule::CommandLine => return Ok(ParseTree::Cmd(Cmd::parse_cmd(pair)?)),
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
        assert_eq!(tree, ParseTree::Help(HelpCmd { command: "aa".to_string() }));
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
