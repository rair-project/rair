use crate::{error_msg, expect, str_to_num, Cmd, Core};
use yansi::Paint;

fn unmap_error(core: &mut Core, name: &str, err: &str) {
    let name = name.primary().bold();
    let msg = format!("Failed to parse {name}, {err}.");
    error_msg(core, "Failed to unmap memory", &msg);
}

#[derive(Default)]
pub struct UnMap;

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
    fn commands(&self) -> &'static [&'static str] {
        &["unmap", "um"]
    }

    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[("[vir] [size]", "Unmap a previosly mapped memory region.")]
    }
}
