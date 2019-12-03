/*
 * files.rs: commands for opening, closing and listing files.
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

#[derive(Default)]
pub struct ListFiles {}

impl ListFiles {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for ListFiles {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 0 {
            expect(core, args.len() as u64, 0);
            return;
        }
        writeln!(core.stdout, "Handle\tStart address\tsize\t\tPermissions\tURI").unwrap();
        for file in core.io.uri_iter() {
            let perm = format!("{:?}", file.perm());
            write!(core.stdout, "{}\t0x{:08x}\t0x{:08x}\t{}", file.hndl(), file.paddr_base(), file.size(), perm).unwrap();
            if perm.len() < 6{
                write!(core.stdout, "\t").unwrap();
            }
            writeln!(core.stdout, "\t{}", file.name()).unwrap();
        }
    }
    fn help(&self, core: &mut Core) {
        help(core, &"files", &"", vec![("", "List all open files.")]);
    }
}
