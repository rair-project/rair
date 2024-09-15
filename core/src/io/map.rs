//! commands for mapping/unmapping memory regions and listing mapped regions as well.

use crate::core::Core;
use crate::helper::{error_msg, expect, help, is_color, str_to_num, Cmd};
use std::io::Write;
use yansi::Paint;

#[derive(Default)]
pub struct Map;

fn map_error(core: &mut Core, name: &str, err: &str) {
    let name = name.primary().bold();
    let msg = format!("Failed to parse {name}, {err}.");
    error_msg(core, "Failed to map memory", &msg);
}
fn unmap_error(core: &mut Core, name: &str, err: &str) {
    let name = name.primary().bold();
    let msg = format!("Failed to parse {name}, {err}.");
    error_msg(core, "Failed to unmap memory", &msg);
}
impl Map {
    pub fn new() -> Self {
        Self
    }
}

impl Cmd for Map {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 3 {
            expect(core, args.len() as u64, 3);
            return;
        }
        let phy = match str_to_num(&args[0]) {
            Ok(p) => p,
            Err(e) => return map_error(core, "phy", &e.to_string()),
        };
        let vir = match str_to_num(&args[1]) {
            Ok(v) => v,
            Err(e) => return map_error(core, "vir", &e.to_string()),
        };
        let size = match str_to_num(&args[2]) {
            Ok(s) => s,
            Err(e) => return map_error(core, "size", &e.to_string()),
        };
        if size == 0 {
            return;
        }
        if let Err(e) = core.io.map(phy, vir, size) {
            error_msg(core, "Failed to map memory", &e.to_string());
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            "map",
            "",
            vec![(
                "[phy] [vir] [size]",
                "Map region from physical address space to virtual address space.",
            )],
        );
    }
}

#[derive(Default)]
pub struct UnMap;

impl UnMap {
    pub fn new() -> Self {
        Self
    }
}

impl Cmd for UnMap {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 2 {
            expect(core, args.len() as u64, 2);
            return;
        }
        let vir = match str_to_num(&args[0]) {
            Ok(v) => v,
            Err(e) => return unmap_error(core, "vir", &e.to_string()),
        };

        let size = match str_to_num(&args[1]) {
            Ok(s) => s,
            Err(e) => return unmap_error(core, "size", &e.to_string()),
        };
        if size == 0 {
            return;
        }
        if let Err(e) = core.io.unmap(vir, size) {
            error_msg(core, "Failed to unmap memory", &e.to_string());
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            "unmap",
            "um",
            vec![("[vir] [size]", "Unmap a previosly mapped memory region.")],
        );
    }
}

#[derive(Default)]
pub struct ListMap;

impl ListMap {
    pub fn new(core: &mut Core) -> Self {
        let env = core.env.clone();
        env.write()
            .add_str_with_cb(
                "maps.headerColor",
                "color.6",
                "Color used in the header of `maps` command",
                core,
                is_color,
            )
            .unwrap();
        Self
    }
}

impl Cmd for ListMap {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if !args.is_empty() {
            expect(core, args.len() as u64, 0);
            return;
        }
        let env = core.env.read();
        let color = env.get_str("maps.headerColor").unwrap();
        let (r, g, b) = env.get_color(color).unwrap();
        writeln!(
            core.stdout,
            "{: <20}{: <20}{}",
            "Virtual Address".rgb(r, g, b),
            "Physical Address".rgb(r, g, b),
            "Size".rgb(r, g, b)
        )
        .unwrap();
        for map in core.io.map_iter() {
            writeln!(
                core.stdout,
                "{: <20}{: <20}0x{:x}",
                format!("0x{:x}", map.vaddr),
                format!("0x{:x}", map.paddr),
                map.size
            )
            .unwrap();
        }
    }
    fn help(&self, core: &mut Core) {
        help(core, "maps", "", vec![("", "List all memory maps.")]);
    }
}
#[cfg(test)]
mod test_mapping {
    use super::*;
    use crate::writer::Writer;
    use rair_io::*;
    use std::path::Path;
    use test_file::*;
    #[test]
    fn test_map_docs() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let map = Map::new();
        map.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Command: [map]\nUsage:\nmap [phy] [vir] [size]\tMap region from physical address space to virtual address space.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_unmap_docs() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let unmap = UnMap::new();
        unmap.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [unmap | um]\nUsage:\num [vir] [size]\tUnmap a previosly mapped memory region.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_list_map_docs() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.help("maps");
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Command: [maps]\nUsage:\nmaps\tList all memory maps.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    fn test_map_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut map = Map::new();
        let mut unmap = UnMap::new();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        map.run(
            &mut core,
            &["0x0".to_owned(), "0x500".to_owned(), "0x20".to_owned()],
        );
        map.run(
            &mut core,
            &["0x10".to_owned(), "0x520".to_owned(), "0x20".to_owned()],
        );
        map.run(
            &mut core,
            &["0x20".to_owned(), "0x540".to_owned(), "0x20".to_owned()],
        );
        map.run(
            &mut core,
            &["0x20".to_owned(), "0x540".to_owned(), "0".to_owned()],
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.run("maps", &[]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Virtual Address     Physical Address    Size\n\
             0x500               0x0                 0x20\n\
             0x520               0x10                0x20\n\
             0x540               0x20                0x20\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        unmap.run(&mut core, &["0x520".to_owned(), "0x20".to_owned()]);
        unmap.run(&mut core, &["0x510".to_owned(), "0x5".to_owned()]);
        unmap.run(&mut core, &["0x510".to_owned(), "0".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.run("maps", &[]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Virtual Address     Physical Address    Size\n\
             0x500               0x0                 0x10\n\
             0x515               0x15                0xb\n\
             0x540               0x20                0x20\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_map() {
        operate_on_file(&test_map_cb, DATA);
    }
    #[test]
    fn test_map_error() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut map = Map::new();
        let mut unmap = UnMap::new();
        map.run(&mut core, &[]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 3 argument(s), found 0.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        map.run(
            &mut core,
            &["08".to_owned(), "0x500".to_owned(), "0x20".to_owned()],
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to map memory\nFailed to parse phy, invalid digit found in string.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        map.run(
            &mut core,
            &["0x0".to_owned(), "0b500".to_owned(), "0x20".to_owned()],
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to map memory\nFailed to parse vir, invalid digit found in string.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        map.run(
            &mut core,
            &["0x0".to_owned(), "0x500".to_owned(), "ff".to_owned()],
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to map memory\nFailed to parse size, invalid digit found in string.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        map.run(
            &mut core,
            &["0x0".to_owned(), "0x500".to_owned(), "0x20".to_owned()],
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to map memory\nCannot resolve address.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.run("maps", &["0xff".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 0 argument(s), found 1.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        unmap.run(&mut core, &["0xff".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 2 argument(s), found 1.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        unmap.run(&mut core, &["0b500".to_owned(), "0x20".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to unmap memory\nFailed to parse vir, invalid digit found in string.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        unmap.run(&mut core, &["0x500".to_owned(), "ff".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to unmap memory\nFailed to parse size, invalid digit found in string.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        unmap.run(&mut core, &["0x500".to_owned(), "0x20".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to unmap memory\nCannot resolve address.\n"
        );
    }
}
