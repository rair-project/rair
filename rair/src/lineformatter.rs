//! Autocompletion / hinting / colorzing input.

use alloc::borrow::Cow::{self, Owned};
use alloc::sync::Arc;
use parking_lot::Mutex;
use rair_cmd::ParseTree;
use rair_core::Commands;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::Context;
use rustyline_derive::{Helper, Validator};
use yansi::Paint as _;

#[derive(Helper, Validator)]
pub struct LineFormatter {
    commands: Arc<Mutex<Commands>>,
    hinter: HistoryHinter,
}

impl LineFormatter {
    pub fn new(commands: Arc<Mutex<Commands>>) -> Self {
        LineFormatter {
            commands,
            hinter: HistoryHinter {},
        }
    }
    fn tree_complete(&self, tree: ParseTree) -> (usize, Vec<Pair>) {
        match tree {
            // If we have a help then we just return
            // all the commands sharing same prefix ending with the help token
            ParseTree::Help(help) => {
                let mut ret = Vec::new();
                let commands = self.commands.lock();
                for suggestion in commands.prefix(&help.command) {
                    let display = (*suggestion).to_owned();
                    let mut replacement = (*suggestion).to_owned();
                    replacement.push('?');
                    ret.push(Pair {
                        display,
                        replacement,
                    });
                }
                (0, ret)
            }
            // if it is command
            // first if we are taking arguments no autocomplate else autocomplete normally ;)
            ParseTree::Cmd(cmd) => {
                if !cmd.args.is_empty() {
                    return (0, Vec::new());
                }
                let mut ret = Vec::new();
                let commands = self.commands.lock();
                for suggestion in commands.prefix(&cmd.command) {
                    let display = (*suggestion).to_owned();
                    let replacement = (*suggestion).to_owned();
                    ret.push(Pair {
                        display,
                        replacement,
                    });
                }
                (0, ret)
            }
            ParseTree::Comment | ParseTree::HelpAll | ParseTree::NewLine => (0, Vec::new()),
        }
    }
}

impl Completer for LineFormatter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        // first figure which token are we completing
        // we will do so by starting at line[pos] and keep incrementing till:
        //  A- we get to see a white space
        //  B- we reach end of text.
        let mut p = pos;
        while p < line.len() {
            let c: Option<char> = line.chars().nth(p);
            if let Some(character) = c {
                if character.is_whitespace() {
                    break;
                }
            }
            p += 1;
        }
        // next we parse the line
        let Some(prefix) = line.get(0..p) else {
            return Ok((0, Vec::new()));
        };
        let t = ParseTree::construct(prefix);
        match t {
            Err(_) => Ok((0, Vec::new())),
            Ok(tree) => Ok(self.tree_complete(tree)),
        }
    }
}

impl Hinter for LineFormatter {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for LineFormatter {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("{}", hint.primary().bold().italic().dim()))
    }
}
