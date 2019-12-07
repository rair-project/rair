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
use self::files::*;
use self::map::*;
use self::print::*;
use core::Core;
use std::cell::RefCell;
use std::rc::Rc;

pub fn register_io(core: &mut Core) {
    core.add_command("map", "", Rc::new(RefCell::new(Map::new())));
    core.add_command("maps", "", Rc::new(RefCell::new(ListMap::new())));
    core.add_command("printHex", "px", Rc::new(RefCell::new(PrintHex::new())));
    core.add_command("unmap", "um", Rc::new(RefCell::new(UnMap::new())));
    core.add_command("files", "", Rc::new(RefCell::new(ListFiles::new())));
    core.add_command("open", "o", Rc::new(RefCell::new(OpenFile::new())));
    core.add_command("close", "", Rc::new(RefCell::new(CloseFile::new())));
}
