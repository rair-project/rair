/*
 * helper.rs: Helper functions for implementing external or internal commands.
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
use yansi::Paint;

pub fn str_to_num(n: &str) -> Result<u64, num::ParseIntError> {
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

pub fn expect(core: &mut Core, args_len: u64, expect: u64) {
    let (r, g, b) = core.color_palette[3];
    let error = Paint::rgb(r, g, b, "Arguments Error").bold();
    let expected = Paint::rgb(r, g, b, format!("{}", expect));
    let found = Paint::rgb(r, g, b, format!("{}", args_len));
    writeln!(core.stderr, "{}: Expected {} argument(s), found {}", error, expected, found).unwrap();
}

pub fn error_msg(core: &mut Core, title: &str, msg: &str) {
    let (r, g, b) = core.color_palette[3];
    writeln!(core.stderr, "{}: {}", Paint::rgb(r, g, b, "Error").bold(), Paint::rgb(r, g, b, title)).unwrap();
    writeln!(core.stderr, "{}", msg).unwrap();
}

pub fn help(core: &mut Core, long: &str, short: &str, usage: Vec<(&str, &str)>) {
    let (r1, g1, b1) = core.color_palette[5];
    let (r2, g2, b2) = core.color_palette[6];
    writeln!(core.stdout, "Commands: [{} | {}]\n", Paint::rgb(r1, g1, b1, long), Paint::rgb(r1, g1, b1, short)).unwrap();
    writeln!(core.stdout, "Usage:").unwrap();
    for (args, description) in usage {
        writeln!(core.stdout, "{} {}\t{}", Paint::rgb(r1, g1, b1, short), Paint::rgb(r2, g2, b2, args), description,).unwrap()
    }
}
