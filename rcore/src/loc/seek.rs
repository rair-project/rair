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

use core::*;
use helper::*;
use std::io::Write;
pub static SEEKFUNCTION: CmdFunctions = CmdFunctions { run: seek_run, help: seek_help };

fn seek_help(core: &mut Core) {
    help(
        core,
        &"seek",
        &"s",
        vec![
            ("+[offset]", "Increase current loc by offset."),
            ("-[offset]", "Decrease current loc by offset."),
            ("[offset]", "Set current location to offset."),
        ],
    );
}

fn seek_run(core: &mut Core, args: &Vec<String>) {
    if args.len() != 1 {
        expect(core, args.len() as u64, 1);
        return;
    }
    if args[0].starts_with("+") {
        match str_to_num(&args[0][1..]) {
            Ok(offset) => core.set_loc(core.get_loc() + offset),
            Err(e) => writeln!(core.stderr, "{}", e.to_string()).unwrap(),
        }
    } else if args[0].starts_with("-") {
        match str_to_num(&args[0][1..]) {
            Ok(offset) => core.set_loc(core.get_loc() - offset),
            Err(e) => writeln!(core.stderr, "{}", e.to_string()).unwrap(),
        }
    } else {
        match str_to_num(&args[0]) {
            Ok(offset) => core.set_loc(offset),
            Err(e) => writeln!(core.stderr, "{}", e.to_string()).unwrap(),
        }
    }
}
