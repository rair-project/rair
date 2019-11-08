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
use core::*;
use helper::*;
use yansi::Paint;
pub static MODEFUNCTION: CmdFunctions = CmdFunctions { run: mode_run, help: mode_help };

fn mode_help(core: &mut Core) {
    help(
        core,
        &"mode",
        &"m",
        vec![("vir", "Set view mode to virtual address space."), ("phy", "Set view mode to phyisical address space.")],
    );
}

fn mode_run(core: &mut Core, args: &Vec<String>) {
    if args.len() != 1 {
        expect(core, args.len() as u64, 1);
        return;
    }
    match &*args[0] {
        "vir" => core.mode = AddrMode::Vir,
        "phy" => core.mode = AddrMode::Phy,
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
