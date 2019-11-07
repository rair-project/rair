/*
 * core.rs: Linking all rair parts together into 1 module.
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

use app_dirs::*;
use rio::*;
use rtrees::bktree::SpellTree;
use rustyline::Editor;
use std::io;
use std::io::Write;
use std::mem;
use std::path::PathBuf;
pub struct CmdFunctions {
    run: &'static fn(&mut Core, &Vec<String>),
    help: &'static fn(),
}

pub enum Writer {
    Write(Box<dyn Write>),
    Bytes(Vec<u8>),
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Writer::Write(writer) => writer.write(buf),
            Writer::Bytes(bytes) => bytes.write(buf),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Writer::Write(writer) => writer.flush(),
            Writer::Bytes(bytes) => bytes.flush(),
        }
    }
}

pub struct Core {
    pub stdout: Writer,
    pub stderr: Writer,
    pub io: RIO,
    pub rl: Editor<()>,
    loc: u64,
    app_info: AppInfo,
    commands: SpellTree<CmdFunctions>,
}
impl Core {
    pub fn new() -> Self {
        let mut core = Core {
            stdout: Writer::Write(Box::new(io::stdout())),
            stderr: Writer::Write(Box::new(io::stderr())),
            io: RIO::new(),
            loc: 0,
            rl: Editor::<()>::new(),
            app_info: AppInfo { name: "rair", author: "RairDevs" },
            commands: SpellTree::new(),
        };
        drop(core.rl.load_history(&core.hist_file()));
        return core;
    }
    pub fn hist_file(&self) -> PathBuf {
        let mut history = app_dir(AppDataType::UserData, &self.app_info, "/").unwrap();
        history.push("history");
        return history;
    }
    pub fn set_loc(&mut self, loc: u64) {
        self.loc = loc;
    }
    pub fn get_loc(&self) -> u64 {
        self.loc
    }
    pub fn add_command(&mut self, command_name: &'static str, functionality: CmdFunctions) {
        // first check that command_name doesn't exist
        let command = command_name.to_string();
        let (exact, _) = self.commands.find(&command, 0);
        if exact.is_empty() {
            self.commands.insert(command, functionality);
        } else {
            writeln!(self.stderr, "Command `{}` already existed", command_name).unwrap();
        }
    }
    pub fn run(&mut self, command: &String, args: &Vec<String>) {
        let (exact, similar) = self.commands.find(&command, 2);
        if exact.is_empty() {
            writeln!(self.stderr, "Command `{}` is not found.", command).unwrap();
            let mut s = similar.iter();
            if let Some(suggestion) = s.next() {
                write!(self.stderr, "Similar command: {}", suggestion).unwrap();
                for suggestion in s {
                    write!(self.stderr, ", {}", suggestion).unwrap();
                }
                writeln!(self.stderr, ".").unwrap();
            }
        } else {
            (exact[1].run)(self, args)
        }
    }
    pub fn run_at(&mut self, command: &String, args: &Vec<String>, at: u64) {
        let old_loc = mem::replace(&mut self.loc, at);
        self.run(command, args);
        self.loc = old_loc;
    }
    pub fn help(&mut self, command: &String) {
        let (exact, similar) = self.commands.find(&command, 2);
        if exact.is_empty() {
            writeln!(self.stderr, "Command `{}` is not found", command).unwrap();
            let mut s = similar.iter();
            if let Some(suggestion) = s.next() {
                write!(self.stderr, "Similar command: {}", suggestion).unwrap();
                for suggestion in s {
                    write!(self.stderr, ", {}", suggestion).unwrap();
                }
                writeln!(self.stderr, "").unwrap();
            }
        } else {
            (exact[1].help)()
        }
    }
}
