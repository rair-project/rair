/*
 * lineformatter.rs: Autocompletion / hinting / colorzing input.
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

use parking_lot::Mutex;
use rair_cmd::*;
use rair_core::Commands;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::Context;
use rustyline_derive::Helper;
use std::borrow::Cow::{self, Owned};
use std::sync::Arc;
use yansi::Paint;

#[derive(Helper)]
pub struct LineFormatter {
    hinter: HistoryHinter,
    commands: Arc<Mutex<Commands>>,
}

impl LineFormatter {
    pub fn new(commands: Arc<Mutex<Commands>>) -> Self {
        LineFormatter { hinter: HistoryHinter {}, commands }
    }
    fn tree_complete(&self, tree: ParseTree) -> Result<(usize, Vec<Pair>), ReadlineError> {
        match tree {
            // If we have a help then we just return
            // all the commands sharing same prefix ending with the help token
            ParseTree::Help(help) => {
                let mut ret = Vec::new();
                for suggestion in self.commands.lock().prefix(&help.command) {
                    let display = (*suggestion).to_string();
                    let replacement = (*suggestion).to_string() + "?";
                    ret.push(Pair { display, replacement });
                }
                return Ok((0, ret));
            }
            // if it is command
            // first if we are taking arguments no autocomplate else autocomplete normally ;)
            ParseTree::Cmd(cmd) => {
                if !cmd.args.is_empty() {
                    return Ok((0, Vec::new()));
                }
                let mut ret = Vec::new();
                for suggestion in self.commands.lock().prefix(&cmd.command) {
                    let display = (*suggestion).to_string();
                    let replacement = (*suggestion).to_string();
                    ret.push(Pair { display, replacement });
                }
                return Ok((0, ret));
            }
            _ => return Ok((0, Vec::new())),
        }
    }
}

impl Completer for LineFormatter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Pair>), ReadlineError> {
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
        let t = ParseTree::construct(&line[0..p]);
        match t {
            Err(_) => return Ok((0, Vec::new())),
            Ok(tree) => return self.tree_complete(tree),
        }
    }
}

impl Hinter for LineFormatter {
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for LineFormatter {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("{}", Paint::default(hint).bold().italic().dimmed()))
    }
}
