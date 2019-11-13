/*
 * quit.rs: Quit the current project.
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
use core::*;
use helper::*;
use std::process;

#[derive(Default)]
pub struct Quit {}

impl Quit {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for Quit {
    fn run(&mut self, _core: &mut Core, _args: &[String]) {
        process::exit(0);
    }
    fn help(&self, core: &mut Core) {
        help(core, &"quit", &"q", vec![("", "Quit Current session.")]);
    }
}
