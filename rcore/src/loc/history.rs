/*
 * history.rs: history managment for seek and mode.
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

use core::Core;
use helper::AddrMode;
#[derive(Default)]
pub struct History {
    back: Vec<(AddrMode, u64)>,
    front: Vec<(AddrMode, u64)>,
}

impl History {
    pub fn new() -> Self {
        return Default::default();
    }
    pub fn backward(&mut self, core: &Core) -> Option<(AddrMode, u64)> {
        let (mode, addr) = self.back.pop()?;
        self.front.push((core.mode, core.get_loc()));
        return Some((mode, addr));
    }
    pub fn forward(&mut self, core: &Core) -> Option<(AddrMode, u64)> {
        let (mode, addr) = self.front.pop()?;
        self.back.push((core.mode, core.get_loc()));
        return Some((mode, addr));
    }
    pub fn add(&mut self, core: &Core) {
        self.front.clear();
        self.back.push((core.mode, core.get_loc()));
    }
}
