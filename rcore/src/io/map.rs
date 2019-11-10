/*
 * map.rs: commands for mapping/unmapping memory regions and listing mapped regions as well.
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
use std::io::Write;
use yansi::Paint;

pub static MAPFUNCTION: CmdFunctions = CmdFunctions { run: map_run, help: map_help };

fn map_help(core: &mut Core) {
    help(core, &"map", &"", vec![("[phy] [vir] [size]", "Map region from physical address space to virtual address space.")]);
}

fn map_run(core: &mut Core, args: &[String]) {
    if args.len() != 3 {
        expect(core, args.len() as u64, 3);
        return;
    }
    let phy;
    let vir;
    let size;
    match str_to_num(&args[0]) {
        Ok(p) => phy = p,
        Err(e) => {
            let name = Paint::default("phy").bold();
            let msg = format!("Failed to parse {}, {}", name, &e.to_string());
            error_msg(core, "Failed to map memory", &msg);
            return;
        }
    }
    match str_to_num(&args[1]) {
        Ok(v) => vir = v,
        Err(e) => {
            let name = Paint::default("vir").bold();
            let msg = format!("Failed to parse {}, {}", name, &e.to_string());
            error_msg(core, "Failed to map memory", &msg);
            return;
        }
    }
    match str_to_num(&args[2]) {
        Ok(s) => size = s,
        Err(e) => {
            let name = Paint::default("size").bold();
            let msg = format!("Failed to parse {}, {}", name, &e.to_string());
            error_msg(core, "Failed to map memory", &msg);
            return;
        }
    }
    if let Err(e) = core.io.map(phy, vir, size) {
        error_msg(core, "Failed to map memory", &e.to_string());
    }
}

pub static UNMAPFUNCTION: CmdFunctions = CmdFunctions { run: unmap_run, help: unmap_help };

fn unmap_help(core: &mut Core) {
    help(core, &"unmap", &"um", vec![("[vir] [size]", "Unmap a previosly mapped memory region.")]);
}

fn unmap_run(core: &mut Core, args: &[String]) {
        if args.len() != 2 {
        expect(core, args.len() as u64, 2);
        return;
    }
    let vir;
    let size;
    match str_to_num(&args[0]) {
        Ok(v) => vir = v,
        Err(e) => {
            let name = Paint::default("phy").bold();
            let msg = format!("Failed to parse {}, {}", name, &e.to_string());
            error_msg(core, "Failed to unmap memory", &msg);
            return;
        }
    }
    match str_to_num(&args[1]) {
        Ok(s) => size = s,
        Err(e) => {
            let name = Paint::default("vir").bold();
            let msg = format!("Failed to parse {}, {}", name, &e.to_string());
            error_msg(core, "Failed to unmap memory", &msg);
            return;
        }
    }
    if let Err(e) = core.io.unmap(vir, size) {
        error_msg(core, "Failed to unmap memory", &e.to_string());
    }
}
pub static LISTMAPFUNCTION: CmdFunctions = CmdFunctions { run: lm_run, help: lm_help };

fn lm_help(core: &mut Core) {
    help(core, &"maps", &"", vec![("", "List all memory maps")]);
}

fn lm_run(core: &mut Core, args: &[String]) {
    if args.len() != 0 {
        expect(core, args.len() as u64, 0);
        return;
    }
    let (r, g, b) = core.color_palette[5];
    writeln!(
        core.stdout,
        "{: <20}{: <20}{: <5}",
        Paint::rgb(r, g, b, "Virtual Address"),
        Paint::rgb(r, g, b, "Physical Address"),
        Paint::rgb(r, g, b, "Size")
    )
    .unwrap();
    for map in core.io.map_iter() {
        writeln!(
            core.stdout,
            "{: <20}{: <20}{: <5}",
            format!("0x{:x}", map.vaddr),
            format!("0x{:x}", map.paddr),
            format!("0x{:x}", map.size)
        )
        .unwrap();
    }
}
