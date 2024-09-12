/*
 * cmd.rs: Main command parsing.
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
use error::*;
use grammar::*;
use pest::iterators::{Pair, Pairs};

#[derive(Debug, PartialEq)]
pub enum RedPipe {
    None,
    Redirect(Box<Argument>),
    RedirectCat(Box<Argument>),
    Pipe(Vec<Argument>),
}

impl Default for RedPipe {
    fn default() -> Self {
        return RedPipe::None;
    }
}
impl RedPipe {
    fn parse_pipe(pairs: Pairs<Rule>) -> Self {
        let mut ret = Vec::new();
        for pair in pairs {
            ret.push(Argument::parse_argument(pair))
        }
        return RedPipe::Pipe(ret);
    }
    fn parse_red(mut pairs: Pairs<Rule>) -> Self {
        let arg = Argument::parse_argument(pairs.next().unwrap());
        return RedPipe::Redirect(Box::new(arg));
    }
    fn parse_redcat(mut pairs: Pairs<Rule>) -> Self {
        let arg = Argument::parse_argument(pairs.next().unwrap());
        return RedPipe::RedirectCat(Box::new(arg));
    }
    fn parse_redpipe(root: Pair<Rule>) -> Self {
        let mut pairs = root.into_inner();
        let type_identifier = pairs.next().unwrap();
        match type_identifier.as_rule() {
            Rule::Pipe => return RedPipe::parse_pipe(pairs),
            Rule::Red => return RedPipe::parse_red(pairs),
            Rule::RedCat => return RedPipe::parse_redcat(pairs),
            _ => unimplemented_pair(type_identifier),
        };
    }
}

#[derive(Debug, PartialEq)]
pub enum Argument {
    NonLiteral(Cmd),
    Literal(String),
    Err(ParserError),
}
impl Argument {
    fn parse_arguments(root: Pair<Rule>) -> Vec<Self> {
        assert_eq!(root.as_rule(), Rule::Arguments);
        let mut args = Vec::new();
        for pair in root.into_inner() {
            args.push(Argument::parse_argument(pair));
        }
        return args;
    }
    fn parse_argument(root: Pair<Rule>) -> Self {
        let arg = root.as_str();
        if arg.starts_with('`') && arg.ends_with('`') {
            let res = Cmd::parse_cmd(root.into_inner().next().unwrap());
            match res {
                Ok(cmd) => return Argument::NonLiteral(cmd),
                Err(e) => return Argument::Err(e),
            }
        } else if arg.starts_with('"') && arg.ends_with('"') {
            return Argument::Literal(arg[1..arg.len() - 1].to_owned());
        } else {
            return Argument::Literal(arg.to_owned());
        }
    }
}
#[derive(Default, Debug, PartialEq)]
pub struct Cmd {
    pub command: String,
    pub args: Vec<Argument>,
    pub loc: Option<u64>,
    pub red_pipe: Box<RedPipe>,
}

fn pair_to_num(root: Pair<Rule>) -> Result<u64, ParserError> {
    let result = match root.as_rule() {
        Rule::BIN => u64::from_str_radix(&root.as_str()[2..], 2),
        Rule::HEX => u64::from_str_radix(&root.as_str()[2..], 16),
        Rule::OCT => u64::from_str_radix(&root.as_str()[1..], 8),
        Rule::DEC => u64::from_str_radix(root.as_str(), 10),
        _ => unimplemented_pair(root),
    };
    match result {
        Ok(x) => return Ok(x),
        Err(e) => return Err(ParserError::Num(e)),
    }
}
impl Cmd {
    pub(crate) fn parse_cmd(root: Pair<Rule>) -> Result<Self, ParserError> {
        assert_eq!(root.as_rule(), Rule::CommandLine);
        let mut cmd: Cmd = Default::default();
        for pair in root.into_inner() {
            match pair.as_rule() {
                Rule::Command => cmd.command = pair.as_str().to_owned(),
                Rule::Loc => cmd.loc = Some(pair_to_num(pair.into_inner().next().unwrap())?),
                Rule::Arguments => cmd.args = Argument::parse_arguments(pair),
                Rule::RedPipe => cmd.red_pipe = Box::new(RedPipe::parse_redpipe(pair)),
                _ => unimplemented_pair(pair),
            }
        }
        return Ok(cmd);
    }
}

#[cfg(test)]
mod test_normal_cmd {
    use super::*;
    use grammar::CliParser;
    use pest::Parser;
    #[test]
    fn test_cmd() {
        let root = CliParser::parse(Rule::CommandLine, "aa").unwrap().next().unwrap();
        let cmd = Cmd::parse_cmd(root).unwrap();
        let mut target: Cmd = Default::default();
        target.command = "aa".to_string();
        assert_eq!(cmd, target);
    }
    #[test]
    fn test_cmd_argument() {
        let root = CliParser::parse(Rule::CommandLine, "aa bb \"cc dd\" `ee ff`").unwrap().next().unwrap();
        let cmd = Cmd::parse_cmd(root).unwrap();
        let mut target: Cmd = Default::default();
        target.command = "aa".to_string();
        target.args.push(Argument::Literal("bb".to_string()));
        target.args.push(Argument::Literal("cc dd".to_string()));
        target.args.push(Argument::NonLiteral(Cmd {
            command: "ee".to_string(),
            args: vec![Argument::Literal("ff".to_string())],
            loc: None,
            red_pipe: Box::new(RedPipe::None),
        }));
        assert_eq!(cmd, target);
    }
    #[test]
    fn test_cmd_argument_bug() {
        let root = CliParser::parse(Rule::CommandLine, "aa bb cc").unwrap().next().unwrap();
        let cmd = Cmd::parse_cmd(root).unwrap();
        let mut target: Cmd = Default::default();
        target.command = "aa".to_string();
        target.args.push(Argument::Literal("bb".to_string()));
        target.args.push(Argument::Literal("cc".to_string()));
        assert_eq!(cmd, target);
    }

    #[test]
    fn test_cmd_loc() {
        let mut root = CliParser::parse(Rule::CommandLine, "aa @ 0x500").unwrap().next().unwrap();
        let mut cmd = Cmd::parse_cmd(root).unwrap();
        let mut target: Cmd = Default::default();
        target.command = "aa".to_string();
        target.loc = Some(0x500);
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa @ 500").unwrap().next().unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.loc = Some(500);
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa @ 0500").unwrap().next().unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.loc = Some(0o500);
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa @ 0500").unwrap().next().unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.loc = Some(0o500);
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa @ 0b10100").unwrap().next().unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.loc = Some(0b10100);
        assert_eq!(cmd, target);
    }

    #[test]
    fn test_cmd_red_pipe() {
        let mut root = CliParser::parse(Rule::CommandLine, "aa | \"/bin/ls\"").unwrap().next().unwrap();
        let mut cmd = Cmd::parse_cmd(root).unwrap();
        let mut target: Cmd = Default::default();
        target.command = "aa".to_string();
        target.red_pipe = Box::new(RedPipe::Pipe(vec![Argument::Literal("/bin/ls".to_string())]));
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa > outfile").unwrap().next().unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.red_pipe = Box::new(RedPipe::Redirect(Box::new(Argument::Literal("outfile".to_string()))));
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa >>outfile").unwrap().next().unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.red_pipe = Box::new(RedPipe::RedirectCat(Box::new(Argument::Literal("outfile".to_string()))));
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa | ls -lah").unwrap().next().unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.red_pipe = Box::new(RedPipe::Pipe(vec![Argument::Literal("ls".to_string()), Argument::Literal("-lah".to_string())]));
        assert_eq!(cmd, target);
    }
}
