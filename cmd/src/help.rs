//! Help part of AST.

use crate::grammar::Rule;
use pest::iterators::Pair;

#[derive(Default, Debug, PartialEq, Eq)]
#[expect(clippy::exhaustive_structs, reason = "parser AST node; closed set")]
pub struct HelpCmd {
    pub command: String,
}

impl HelpCmd {
    pub(crate) fn parse_help(root: Pair<Rule>) -> Self {
        debug_assert_eq!(
            root.as_rule(),
            Rule::HelpLine,
            "parse_help must be called with a HelpLine pair"
        );
        Self {
            command: root.into_inner().next().unwrap().as_str().to_owned(),
        }
    }
}

#[cfg(test)]
mod test_help_cmd {
    use super::*;
    use crate::grammar::CliParser;
    use pest::Parser as _;
    #[test]
    fn help_no_space() {
        let root = CliParser::parse(Rule::HelpLine, "aa?")
            .unwrap()
            .next()
            .unwrap();
        let help = HelpCmd::parse_help(root);
        assert_eq!(
            help,
            HelpCmd {
                command: "aa".to_owned()
            }
        );
    }
    #[test]
    fn help_space() {
        let root = CliParser::parse(Rule::HelpLine, "aa          ?")
            .unwrap()
            .next()
            .unwrap();
        let help = HelpCmd::parse_help(root);
        assert_eq!(
            help,
            HelpCmd {
                command: "aa".to_owned()
            }
        );
    }
}
