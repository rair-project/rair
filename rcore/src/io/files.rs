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
use rio::*;

#[derive(Default)]
pub struct ListFiles {}

impl ListFiles {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for ListFiles {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if !args.is_empty() {
            expect(core, args.len() as u64, 0);
            return;
        }
        writeln!(core.stdout, "Handle\tStart address\tsize\t\tPermissions\tURI").unwrap();
        for file in core.io.uri_iter() {
            let perm = format!("{:?}", file.perm());
            write!(core.stdout, "{}\t0x{:08x}\t0x{:08x}\t{}", file.hndl(), file.paddr_base(), file.size(), perm).unwrap();
            if perm.len() < 6 {
                write!(core.stdout, "\t").unwrap();
            }
            writeln!(core.stdout, "\t{}", file.name()).unwrap();
        }
    }
    fn help(&self, core: &mut Core) {
        help(core, &"files", &"", vec![("", "List all open files.")]);
    }
}

#[derive(Default)]
pub struct OpenFile {}

impl OpenFile {
    pub fn new() -> Self {
        Default::default()
    }
}
fn parse_perm(p: &str) -> Result<IoMode,String> {
    let mut perm = Default::default();
    for c in p.to_lowercase().chars() {
        match c {
            'r' => perm |= IoMode::READ,
            'w' => perm |= IoMode::WRITE,
            'c' => perm |= IoMode::COW,
            _ => return Err(format!("Unknown Permission: `{}`", c)),
        }
    }
    return Ok(perm);
}
impl Cmd for OpenFile {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() > 3 || args.len() < 1 {
            expect_range(core, args.len() as u64, 1, 2);
            return;
        }
        let uri;
        let perm;
        let addr;
        if args.len() == 3 {
            uri = &args[1];
            perm = match parse_perm(&args[0]) {
                Ok(perm) => perm,
                Err(e) => return error_msg(core, "Failed to parse permission",&e),
            };
            addr = match str_to_num(&args[2]) {
                Ok(addr) => Some(addr),
                Err(e) => {
                    let err_str = format!("{}", e);
                    error_msg(core, "Failed to parse address", &err_str);
                    return;
                }
            }
        }
        else if args.len() == 2 {
            if let Ok(a) = str_to_num(&args[1]) {
                addr = Some(a);
                uri = &args[0];
                perm = IoMode::READ;
            } else {
                uri = &args[1];
                perm = match parse_perm(&args[0]) {
                    Ok(perm) => perm,
                    Err(e) => return error_msg(core, "Failed to parse permission",&e),
                };
                addr = None;
            }
                
        } else {
            uri = &args[0];
            perm = IoMode::READ;
            addr = None;
        }

        let result = match addr {
            Some(addr) => core.io.open_at(uri, perm, addr),
            None => core.io.open(uri, perm),
        };
        if let Err(e) = result {
            let err_str = format!("{}", e);
            error_msg(core, "Failed to Open File", &err_str);
        }

    }
    fn help(&self, core: &mut Core) {
        help(core, &"open", &"o", vec![("<Perm> [URI] <Addr>", "Open given URI using given optional permission (default to readonly) at given optional address.")]);
    }
}

#[derive(Default)]
pub struct CloseFile {}


impl CloseFile {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for CloseFile {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return;
        }
        let hndl =  match str_to_num(&args[0]){
            Ok(hndl) => hndl,
            Err(e) => {
                let err_str = format!("{}", e);
                error_msg(core, "Invalid hndl", &err_str);
                return;

            }
        };
        if let Err(e) = core.io.close(hndl) {
            let err_str = format!("{}", e);
            error_msg(core, "Failed to Open File", &err_str);
        }

    }
    fn help(&self, core: &mut Core) {
        help(core, &"close", &"", vec![("[hndl]", "Close file with given hndl.")]);
    }
}

