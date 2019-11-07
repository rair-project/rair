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
use std::io::Write;
use std::num;
pub static SEEKFUNCTON : CmdFunctions = CmdFunctions {
    run: seek_run,
    help: seek_help,
};

fn seek_help(core: &mut Core) {
    writeln!(core.stdout, "Seek Command: [seek | s]").unwrap();
    writeln!(core.stdout, "Usage:").unwrap();
    writeln!(core.stdout, "s +offset\t Increase current loc by offset.").unwrap();
    writeln!(core.stdout, "s -offset\t Decrease current loc by offset.").unwrap();
    writeln!(core.stdout, "s offset\t Set current location to offset.").unwrap();
}
fn str_to_num(n: &str) -> Result<u64, num::ParseIntError> {
    if n.len() >= 2 {
        match &*n[0..2].to_lowercase() {
            "0b" => return u64::from_str_radix(&n[2..], 2),
            "0x" => return u64::from_str_radix(&n[2..], 16),
            _ => (),
        }
    }
    if n.chars().nth(0).unwrap() == '0' {
        return u64::from_str_radix(&n[1..], 8);
    }
    return u64::from_str_radix(n, 10);
}

fn seek_run(core: &mut Core, args: &Vec<String>) {
    if args.len() != 1 {
        writeln!(core.stderr, "Expected 1 argument found {}", args.len()).unwrap();
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