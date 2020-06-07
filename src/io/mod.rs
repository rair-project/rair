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
use crate::core::Core;
use parking_lot::Mutex;
use std::sync::Arc;
pub fn register_io(core: &mut Core) {
    let maps = Arc::new(Mutex::new(ListMap::new(core)));
    let files = Arc::new(Mutex::new(ListFiles::new(core)));
    let px = Arc::new(Mutex::new(PrintHex::new(core)));

    core.add_command("map", "", Arc::new(Mutex::new(Map::new())));
    core.add_command("maps", "", maps);
    core.add_command("printHex", "px", px);
    core.add_command("printBase", "pb", Arc::new(Mutex::new(PrintBase::new())));
    core.add_command("printCSV", "pcsv", Arc::new(Mutex::new(PrintCSV::new())));
    core.add_command("printSignedCSV", "pscsv", Arc::new(Mutex::new(PrintSignedCSV::new())));
    core.add_command("unmap", "um", Arc::new(Mutex::new(UnMap::new())));
    core.add_command("files", "", files);
    core.add_command("open", "o", Arc::new(Mutex::new(OpenFile::new())));
    core.add_command("close", "", Arc::new(Mutex::new(CloseFile::new())));
    core.add_command("writeHex", "wx", Arc::new(Mutex::new(WriteHex::new())));
    core.add_command("writeToFile", "wtf", Arc::new(Mutex::new(WriteToFile::new())));
}
