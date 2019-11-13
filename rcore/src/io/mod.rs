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
mod map;
mod print_hex;
use self::map::*;
use self::print_hex::*;
use core::Core;

pub fn register_io(core: &mut Core) {
    core.add_command("map", Box::new(Map::new()));
    core.add_command("maps", Box::new(ListMap::new()));
    core.add_command("printHex", Box::new(PrintHex::new()));
    core.add_command("px", Box::new(PrintHex::new()));
    core.add_command("unmap", Box::new(UnMap::new()));
    core.add_command("um", Box::new(UnMap::new()));
}
