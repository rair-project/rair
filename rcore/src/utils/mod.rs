/*
 * utils: Utility commands.
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
mod quit;

pub use self::quit::Quit;
use core::Core;
use std::cell::RefCell;
use std::rc::Rc;

pub fn register_utils(core: &mut Core) {
    core.add_command("quit", "q", Rc::new(RefCell::new(Quit::new())));
}
