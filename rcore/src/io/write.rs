/*
 * write: commands handling data writing to files.
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
use std::fs::File;
use std::io::prelude::*;
use std::u8;

#[derive(Default)]
pub struct WriteHex {}

impl WriteHex {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for WriteHex {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return;
        }
        if args[0].len() % 2 != 0 {
            error_msg(core, "Failed to hexpairs", "Data can't have odd number of digits.");
            return;
        }
        let mut hexpairs = args[0].chars().peekable();
        let mut data = Vec::with_capacity(args.len() / 2);
        while hexpairs.peek().is_some() {
            let chunk: String = hexpairs.by_ref().take(2).collect();
            let byte = match u8::from_str_radix(&chunk, 16) {
                Ok(byte) => byte,
                Err(e) => {
                    let msg = format!("{}", e);
                    return error_msg(core, "Failed to hexpairs", &msg);
                }
            };
            data.push(byte);
        }
        let loc = core.get_loc();
        let error = match core.mode {
            AddrMode::Phy => core.io.pwrite(loc, &data),
            AddrMode::Vir => core.io.vwrite(loc, &data),
        };
        if let Err(e) = error {
            error_msg(core, "Read Failed", &e.to_string());
            return;
        }
    }
    fn help(&self, core: &mut Core) {
        help(core, &"writetHex", &"wx", vec![("[hexpairs]", "write given hexpairs data into the current address.")]);
    }
}

#[derive(Default)]
pub struct WriteToFile {}

impl WriteToFile {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for WriteToFile {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 2 {
            expect(core, args.len() as u64, 2);
            return;
        }
        let size = match str_to_num(&args[0]) {
            Ok(size) => size as usize,
            Err(e) => {
                let err_str = format!("{}", e);
                error_msg(core, "Failed to parse size", &err_str);
                return;
            }
        };
        let loc = core.get_loc();
        let mut data = vec![0; size];
        let error = match core.mode {
            AddrMode::Phy => core.io.pread(loc, &mut data),
            AddrMode::Vir => core.io.vread(loc, &mut data),
        };
        if let Err(e) = error {
            error_msg(core, "Failed to read data", &e.to_string());
            return;
        }
        let mut file = match File::create(&args[1]) {
            Ok(file) => file,
            Err(e) => {
                let err_str = format!("{}", e);
                error_msg(core, "Failed to open file", &err_str);
                return;
            }
        };
        if let Err(e) = file.write_all(&data) {
            let err_str = format!("{}", e);
            error_msg(core, "Failed to write data to file", &err_str);
            return;
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            &"writeToFile",
            &"wtf",
            vec![("[size] [filepath]", "write data of size [size] at current location to file identified by [filepath].")],
        );
    }
}
