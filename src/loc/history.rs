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
        Default::default()
    }
    pub fn backward(&mut self, core: &Core) -> Option<(AddrMode, u64)> {
        let (mode, addr) = self.back.pop()?;
        self.front.push((core.mode, core.get_loc()));
        Some((mode, addr))
    }
    pub fn forward(&mut self, core: &Core) -> Option<(AddrMode, u64)> {
        let (mode, addr) = self.front.pop()?;
        self.back.push((core.mode, core.get_loc()));
        Some((mode, addr))
    }
    pub fn add(&mut self, core: &Core) {
        self.front.clear();
        self.back.push((core.mode, core.get_loc()));
    }
}

#[cfg(test)]
mod test_history {
    use super::*;
    #[test]
    fn test_history() {
        let mut history = History::new();
        let mut core = Core::new_no_colors();
        assert_eq!(history.backward(&core), None);
        assert_eq!(history.backward(&core), None);
        history.add(&core);
        core.set_loc(0x50);
        history.add(&core);
        core.set_loc(0x100);
        core.mode = AddrMode::Vir;
        history.add(&core);
        core.set_loc(0x150);
        core.mode = AddrMode::Phy;
        history.add(&core);
        assert_eq!(history.backward(&core).unwrap(), (AddrMode::Phy, 0x150));
        core.set_loc(0x150);
        assert_eq!(history.backward(&core).unwrap(), (AddrMode::Vir, 0x100));
        core.set_loc(0x100);
        core.mode = AddrMode::Vir;
        assert_eq!(history.backward(&core).unwrap(), (AddrMode::Phy, 0x50));
        core.set_loc(0x50);
        core.mode = AddrMode::Phy;
        assert_eq!(history.forward(&core).unwrap(), (AddrMode::Vir, 0x100));
        core.set_loc(0x100);
        core.mode = AddrMode::Vir;
        assert_eq!(history.backward(&core).unwrap(), (AddrMode::Phy, 0x50));
        core.set_loc(0x50);
        core.mode = AddrMode::Phy;
        assert_eq!(history.backward(&core).unwrap(), (AddrMode::Phy, 0x0));
        core.set_loc(0x0);
        core.mode = AddrMode::Phy;
        assert_eq!(history.backward(&core), None);
        assert_eq!(history.forward(&core).unwrap(), (AddrMode::Phy, 0x50));
        assert_eq!(history.forward(&core).unwrap(), (AddrMode::Vir, 0x100));
        assert_eq!(history.forward(&core).unwrap(), (AddrMode::Phy, 0x150));
    }
}
