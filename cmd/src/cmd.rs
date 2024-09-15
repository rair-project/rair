//! Main command parsing.

use crate::{
    error::{unimplemented_pair, ParserError},
    grammar::Rule,
};
use pest::iterators::{Pair, Pairs};

#[derive(Debug, PartialEq, Default)]
pub enum RedPipe {
    #[default]
    None,
    Redirect(Box<Argument>),
    RedirectCat(Box<Argument>),
    Pipe(Vec<Argument>),
}

impl RedPipe {
    fn parse_pipe(pairs: Pairs<Rule>) -> Self {
        let mut ret = Vec::new();
        for pair in pairs {
            ret.push(Argument::parse_argument(pair));
        }
        Self::Pipe(ret)
    }
    fn parse_red(mut pairs: Pairs<Rule>) -> Self {
        let arg = Argument::parse_argument(pairs.next().unwrap());
        Self::Redirect(Box::new(arg))
    }
    fn parse_redcat(mut pairs: Pairs<Rule>) -> Self {
        let arg = Argument::parse_argument(pairs.next().unwrap());
        Self::RedirectCat(Box::new(arg))
    }
    fn parse_redpipe(root: Pair<Rule>) -> Self {
        let mut pairs = root.into_inner();
        let type_identifier = pairs.next().unwrap();
        match type_identifier.as_rule() {
            Rule::Pipe => Self::parse_pipe(pairs),
            Rule::Red => Self::parse_red(pairs),
            Rule::RedCat => Self::parse_redcat(pairs),
            Rule::EOI
            | Rule::HelpAll
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
            | Rule::RedPipe
            | Rule::Comment
            | Rule::EmptyLine
            | Rule::HelpLine
            | Rule::CommandLine
            | Rule::Input => unimplemented_pair(&type_identifier),
        }
    }
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Argument {
    NonLiteral(Cmd),
    Literal(String),
    Err(Box<ParserError>),
}
impl Argument {
    fn parse_arguments(root: Pair<Rule>) -> Vec<Self> {
        assert_eq!(root.as_rule(), Rule::Arguments);
        let mut args = Vec::new();
        for pair in root.into_inner() {
            args.push(Self::parse_argument(pair));
        }
        args
    }
    fn parse_argument(root: Pair<Rule>) -> Self {
        let arg = root.as_str();
        if arg.starts_with('`') && arg.ends_with('`') {
            let res = Cmd::parse_cmd(root.into_inner().next().unwrap());
            match res {
                Ok(cmd) => Self::NonLiteral(cmd),
                Err(e) => Self::Err(Box::new(e)),
            }
        } else if arg.starts_with('"') && arg.ends_with('"') {
            return Self::Literal(arg[1..arg.len() - 1].to_owned());
        } else {
            return Self::Literal(arg.to_owned());
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

fn pair_to_num(root: &Pair<Rule>) -> Result<u64, ParserError> {
    let result = match root.as_rule() {
        Rule::BIN => u64::from_str_radix(&root.as_str()[2..], 2),
        Rule::HEX => u64::from_str_radix(&root.as_str()[2..], 16),
        Rule::OCT => u64::from_str_radix(&root.as_str()[1..], 8),
        Rule::DEC => root.as_str().parse::<u64>(),
        Rule::EOI
        | Rule::WHITESPACE
        | Rule::CustomAlpha
        | Rule::CustomAlphaNum
        | Rule::ANS
        | Rule::ANWS
        | Rule::Command
        | Rule::ArgumentLiteral
        | Rule::Argument
        | Rule::Arguments
        | Rule::Loc
        | Rule::Pipe
        | Rule::Red
        | Rule::RedCat
        | Rule::RedPipe
        | Rule::Comment
        | Rule::EmptyLine
        | Rule::HelpLine
        | Rule::CommandLine
        | Rule::Input
        | Rule::HelpAll => unimplemented_pair(root),
    };
    match result {
        Ok(x) => Ok(x),
        Err(e) => Err(ParserError::Num(e)),
    }
}
impl Cmd {
    pub(crate) fn parse_cmd(root: Pair<Rule>) -> Result<Self, ParserError> {
        assert_eq!(root.as_rule(), Rule::CommandLine);
        let mut cmd = Cmd::default();
        for pair in root.into_inner() {
            match pair.as_rule() {
                Rule::Command => pair.as_str().clone_into(&mut cmd.command),
                Rule::Loc => cmd.loc = Some(pair_to_num(&pair.into_inner().next().unwrap())?),
                Rule::Arguments => cmd.args = Argument::parse_arguments(pair),
                Rule::RedPipe => cmd.red_pipe = Box::new(RedPipe::parse_redpipe(pair)),
                Rule::EOI
                | Rule::HelpAll
                | Rule::WHITESPACE
                | Rule::CustomAlpha
                | Rule::CustomAlphaNum
                | Rule::ANS
                | Rule::ANWS
                | Rule::DEC
                | Rule::BIN
                | Rule::HEX
                | Rule::OCT
                | Rule::ArgumentLiteral
                | Rule::Argument
                | Rule::Pipe
                | Rule::Red
                | Rule::RedCat
                | Rule::Comment
                | Rule::EmptyLine
                | Rule::HelpLine
                | Rule::CommandLine
                | Rule::Input => unimplemented_pair(&pair),
            }
        }
        Ok(cmd)
    }
}

#[cfg(test)]
mod test_normal_cmd {
    use super::*;
    use crate::grammar::CliParser;
    use pest::Parser;
    #[test]
    fn test_cmd() {
        let root = CliParser::parse(Rule::CommandLine, "aa")
            .unwrap()
            .next()
            .unwrap();
        let cmd = Cmd::parse_cmd(root).unwrap();
        let target = Cmd {
            command: "aa".to_owned(),
            ..Default::default()
        };
        assert_eq!(cmd, target);
    }
    #[test]
    fn test_cmd_argument() {
        let root = CliParser::parse(Rule::CommandLine, "aa bb \"cc dd\" `ee ff`")
            .unwrap()
            .next()
            .unwrap();
        let cmd = Cmd::parse_cmd(root).unwrap();
        let target = Cmd {
            command: "aa".to_owned(),
            args: vec![
                Argument::Literal("bb".to_owned()),
                Argument::Literal("cc dd".to_owned()),
                Argument::NonLiteral(Cmd {
                    command: "ee".to_owned(),
                    args: vec![Argument::Literal("ff".to_owned())],
                    loc: None,
                    red_pipe: Box::new(RedPipe::None),
                }),
            ],
            ..Default::default()
        };
        assert_eq!(cmd, target);
    }
    #[test]
    fn test_cmd_argument_bug() {
        let root = CliParser::parse(Rule::CommandLine, "aa bb cc")
            .unwrap()
            .next()
            .unwrap();
        let cmd = Cmd::parse_cmd(root).unwrap();
        let target = Cmd {
            command: "aa".to_owned(),
            args: vec![
                Argument::Literal("bb".to_owned()),
                Argument::Literal("cc".to_owned()),
            ],
            ..Default::default()
        };
        assert_eq!(cmd, target);
    }

    #[test]
    fn test_cmd_loc() {
        let mut root = CliParser::parse(Rule::CommandLine, "aa @ 0x500")
            .unwrap()
            .next()
            .unwrap();
        let mut cmd = Cmd::parse_cmd(root).unwrap();
        let mut target = Cmd {
            command: "aa".to_owned(),
            loc: Some(0x500),
            ..Default::default()
        };
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa @ 500")
            .unwrap()
            .next()
            .unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.loc = Some(500);
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa @ 0500")
            .unwrap()
            .next()
            .unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.loc = Some(0o500);
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa @ 0500")
            .unwrap()
            .next()
            .unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.loc = Some(0o500);
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa @ 0b10100")
            .unwrap()
            .next()
            .unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.loc = Some(0b10100);
        assert_eq!(cmd, target);
    }

    #[test]
    fn test_cmd_red_pipe() {
        let mut root = CliParser::parse(Rule::CommandLine, "aa | \"/bin/ls\"")
            .unwrap()
            .next()
            .unwrap();
        let mut cmd = Cmd::parse_cmd(root).unwrap();
        let mut target = Cmd {
            command: "aa".to_owned(),
            red_pipe: Box::new(RedPipe::Pipe(vec![Argument::Literal("/bin/ls".to_owned())])),
            ..Default::default()
        };
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa > outfile")
            .unwrap()
            .next()
            .unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.red_pipe = Box::new(RedPipe::Redirect(Box::new(Argument::Literal(
            "outfile".to_owned(),
        ))));
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa >>outfile")
            .unwrap()
            .next()
            .unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.red_pipe = Box::new(RedPipe::RedirectCat(Box::new(Argument::Literal(
            "outfile".to_owned(),
        ))));
        assert_eq!(cmd, target);

        root = CliParser::parse(Rule::CommandLine, "aa | ls -lah")
            .unwrap()
            .next()
            .unwrap();
        cmd = Cmd::parse_cmd(root).unwrap();
        target.red_pipe = Box::new(RedPipe::Pipe(vec![
            Argument::Literal("ls".to_owned()),
            Argument::Literal("-lah".to_owned()),
        ]));
        assert_eq!(cmd, target);
    }
}
