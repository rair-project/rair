/*
 * parser.rs: Generate AST like structure for rair commands.
 * Copyright (C) 2019  Oddcoder
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use pest::Parser;
use help::HelpCmd;
use grammar::*;
use error::ParserError;
use cmd::Cmd;

#[derive(Debug)]
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
        for pair in pairs.unwrap().next().unwrap().into_inner() {
            match pair.as_rule() {
                Rule::HelpLine => return Ok(ParseTree::Help(HelpCmd::parse_help(pair))),
                Rule::Comment => return Ok(ParseTree::Comment),
                Rule::EmptyLine => return Ok(ParseTree::NewLine),
                Rule::CommandLine => return Ok(ParseTree::Cmd(Cmd::parse_cmd(pair)?)),
                _ => {
                    println!("{:#?}", pair);
                    unimplemented!();
                }
            }
        }
        return Ok(ParseTree::Comment);
    }
}
