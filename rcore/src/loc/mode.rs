/*
 * mode.rs: commands handling view mode (phy/vir).
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
use super::history::History;
use core::*;
use helper::*;
use yansi::Paint;

#[derive(Default)]
pub struct Mode {
    history: MRc<History>,
}

impl Mode {
    pub fn new() -> Self {
        Default::default()
    }
    pub(super) fn with_history(history: MRc<History>) -> Self {
        Mode { history }
    }
}

impl Cmd for Mode {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return;
        }
        match &*args[0] {
            "vir" => {
                self.history.borrow_mut().add(core);
                core.mode = AddrMode::Vir;
            }
            "phy" => {
                self.history.borrow_mut().add(core);
                core.mode = AddrMode::Phy
            }
            _ => {
                let msg = format!(
                    "Expected {} or {}, but found {}",
                    Paint::default("vir").italic().bold(),
                    Paint::default("phy").italic().bold(),
                    Paint::default(&args[0]).italic().bold(),
                );
                error_msg(core, "Invalid Mode", &msg);
            }
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            &"mode",
            &"m",
            vec![("vir", "Set view mode to virtual address space."), ("phy", "Set view mode to phyisical address space.")],
        );
    }
}
