use crate::{error_msg, expect, str_to_num, Cmd, Core};
use yansi::Paint;

#[derive(Default)]
pub struct Map;

fn map_error(core: &mut Core, name: &str, err: &str) {
    let name = name.primary().bold();
    let msg = format!("Failed to parse {name}, {err}.");
    error_msg(core, "Failed to map memory", &msg);
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
    fn commands(&self) -> &'static [&'static str] {
        &["map"]
    }

    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[(
            "[phy] [vir] [size]",
            "Map region from physical address space to virtual address space.",
        )]
    }
}
