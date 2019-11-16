/*
 * seek.rs: seek forward or backward in file.
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

use super::history::History;
use core::*;
use helper::*;

#[derive(Default)]
pub struct Seek {
    history: MRc<History>,
}

impl Seek {
    pub fn new() -> Self {
        Default::default()
    }
    pub(super) fn with_history(history: MRc<History>) -> Self {
        Seek { history }
    }
}

impl Cmd for Seek {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return;
        }
        if args[0] == "-" {
            match self.history.borrow_mut().backward(core) {
                Some((mode, addr)) => {
                    core.mode = mode;
                    core.set_loc(addr);
                }
                None => error_msg(core, "Seek Error", "History is empty."),
            }
        } else if args[0] == "+" {
            match self.history.borrow_mut().forward(core) {
                Some((mode, addr)) => {
                    core.mode = mode;
                    core.set_loc(addr);
                }
                None => error_msg(core, "Seek Error", "History is empty."),
            }
        } else if args[0].starts_with('+') {
            match str_to_num(&args[0][1..]) {
                Ok(offset) => {
                    self.history.borrow_mut().add(core);
                    core.set_loc(core.get_loc() + offset);
                }
                Err(e) => error_msg(core, "Seek Error", &e.to_string()),
            }
        } else if args[0].starts_with('-') {
            match str_to_num(&args[0][1..]) {
                Ok(offset) => {
                    self.history.borrow_mut().add(core);
                    core.set_loc(core.get_loc() - offset);
                }
                Err(e) => error_msg(core, "Seek Error", &e.to_string()),
            }
        } else {
            match str_to_num(&args[0]) {
                Ok(offset) => {
                    self.history.borrow_mut().add(core);
                    core.set_loc(offset);
                }
                Err(e) => error_msg(core, "Seek Error", &e.to_string()),
            }
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            &"seek",
            &"s",
            vec![
                ("+", "\tRedo Seek."),
                ("-", "\tUndo Seek."),
                ("+[offset]", "Increase current loc by offset."),
                ("-[offset]", "Decrease current loc by offset."),
                ("[offset]", "Set current location to offset."),
            ],
        );
    }
}
