//! Help part of AST.

use crate::grammar::Rule;
use pest::iterators::Pair;

#[derive(Default, Debug, PartialEq)]
pub struct HelpCmd {
    pub command: String,
}

impl HelpCmd {
    pub(crate) fn parse_help(root: Pair<Rule>) -> Self {
        assert_eq!(root.as_rule(), Rule::HelpLine);
        HelpCmd {
            command: root.into_inner().next().unwrap().as_str().to_owned(),
        }
    }
}

#[cfg(test)]
mod test_help_cmd {
    use super::*;
    use crate::grammar::CliParser;
    use pest::Parser;
    #[test]
    fn test_help_no_space() {
        let root = CliParser::parse(Rule::HelpLine, "aa?")
            .unwrap()
            .next()
            .unwrap();
        let help = HelpCmd::parse_help(root);
        assert_eq!(
            help,
            HelpCmd {
                command: "aa".to_string()
            }
        );
    }
    #[test]
    fn test_help_space() {
        let root = CliParser::parse(Rule::HelpLine, "aa          ?")
            .unwrap()
            .next()
            .unwrap();
        let help = HelpCmd::parse_help(root);
        assert_eq!(
            help,
            HelpCmd {
                command: "aa".to_string()
            }
        );
    }
}
