//! history managment for seek and mode.

use crate::core::Core;
use crate::helper::AddrMode;
#[derive(Default)]
pub struct History {
    back: Vec<(AddrMode, u64)>,
    front: Vec<(AddrMode, u64)>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
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
