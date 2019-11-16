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
                if core.mode == AddrMode::Phy {
                    let vir = core.io.phy_to_vir(core.get_loc());
                    if !vir.is_empty() {
                        core.set_loc(vir[0]);
                    }
                }
                core.mode = AddrMode::Vir;
            }
            "phy" => {
                self.history.borrow_mut().add(core);
                if core.mode == AddrMode::Vir {
                    if let Some(vir) = core.io.vir_to_phy(core.get_loc(), 1) {
                        core.set_loc(vir[0].paddr);
                    }
                }
                core.mode = AddrMode::Phy
            }
            _ => {
                let msg = format!(
                    "Expected {} or {}, but found {}.",
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
            vec![("vir", "Set view mode to virtual address space."), ("phy", "Set view mode to physical address space.")],
        );
    }
}

#[cfg(test)]
mod test_mode {
    use super::*;
    use writer::Writer;
    use yansi::Paint;
    use test_file::*;
    use std::path::Path;
    use rio::*;

    #[test]
    fn test_docs() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mode: Mode = Default::default();
        mode.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [mode | m]\n\
             \n\
             Usage:\n\
             m vir\tSet view mode to virtual address space.\n\
             m phy\tSet view mode to physical address space.\n\
             "
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    fn test_mode_cb(path: &Path) {
        let mut core = Core::new();
        let len = DATA.len() as u64;
        let mut mode: Mode = Default::default();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.io.map(0x0, 0x5000, len).unwrap();
        assert_eq!(core.get_loc(), 0x0);
        mode.run(&mut core, &["vir".to_string()]);
        assert_eq!(core.get_loc(), 0x5000);
        assert_eq!(core.mode, AddrMode::Vir);
        core.set_loc(0x5001);
        mode.run(&mut core, &["phy".to_string()]);
        assert_eq!(core.get_loc(), 1);
        assert_eq!(core.mode, AddrMode::Phy);
        core.set_loc(len + 10);
        mode.run(&mut core, &["vir".to_string()]);
        assert_eq!(core.get_loc(), len + 10);
        assert_eq!(core.mode, AddrMode::Vir);
        mode.run(&mut core, &["phy".to_string()]);
        assert_eq!(core.get_loc(), len + 10);
        assert_eq!(core.mode, AddrMode::Phy);
        
        
    }
    #[test]
    fn test_mode() {
        operate_on_file(&test_mode_cb, DATA);
    }

    #[test]
    fn test_mode_errors() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut mode: Mode = Default::default();
        mode.run(&mut core, &[]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "Arguments Error: Expected 1 argument(s), found 0.\n");

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        mode.run(&mut core, &["not_real_arg".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "Error: Invalid Mode\nExpected vir or phy, but found not_real_arg.\n");

    }
}
