/*
 * io: commands handling IO.
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
mod files;
mod map;
mod print;
mod write;

use self::files::*;
use self::map::*;
use self::print::*;
use self::write::*;
use core::Core;
use std::cell::RefCell;
use std::rc::Rc;

pub fn register_io(core: &mut Core) {
    let maps = Rc::new(RefCell::new(ListMap::new(core)));
    let files = Rc::new(RefCell::new(ListFiles::new(core)));
    let px = Rc::new(RefCell::new(PrintHex::new(core)));

    core.add_command("map", "", Rc::new(RefCell::new(Map::new())));
    core.add_command("maps", "", maps);
    core.add_command("printHex", "px", px);
    core.add_command("printBase", "pb", Rc::new(RefCell::new(PrintBase::new())));
    core.add_command("printCSV", "pcsv", Rc::new(RefCell::new(PrintCSV::new())));
    core.add_command("printSignedCSV", "pscsv", Rc::new(RefCell::new(PrintSignedCSV::new())));
    core.add_command("unmap", "um", Rc::new(RefCell::new(UnMap::new())));
    core.add_command("files", "", files);
    core.add_command("open", "o", Rc::new(RefCell::new(OpenFile::new())));
    core.add_command("close", "", Rc::new(RefCell::new(CloseFile::new())));
    core.add_command("writeHex", "wx", Rc::new(RefCell::new(WriteHex::new())));
    core.add_command("writeToFile", "wtf", Rc::new(RefCell::new(WriteToFile::new())));
}
