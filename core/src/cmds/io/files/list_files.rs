use yansi::Paint;

use crate::{expect, is_color, Cmd, Core};
use std::io::Write;

#[derive(Default)]
pub struct ListFiles;

impl ListFiles {
    pub fn new(core: &mut Core) -> Self {
        //TODO instead of hardcoding command name use it from [`Cmd`]
        let env = core.env.clone();
        env.write()
            .add_str_with_cb(
                "files.headerColor",
                "color.6",
                "Color used in the header of `files` command",
                core,
                is_color,
            )
            .unwrap();
        Self
    }
}

impl Cmd for ListFiles {
    fn commands(&self) -> &'static [&'static str] {
        &["files"]
    }
    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[("", "List all open files.")]
    }
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
            "{}",
            "Handle\tStart address\tsize\t\tPermissions\tURI".rgb(r, g, b)
        )
        .unwrap();
        for file in core.io.uri_iter() {
            let perm = format!("{}", file.perm());
            write!(
                core.stdout,
                "{}\t0x{:08x}\t0x{:08x}\t{}",
                file.hndl(),
                file.paddr_base(),
                file.size(),
                perm
            )
            .unwrap();
            if perm.len() < 6 {
                write!(core.stdout, "\t").unwrap();
            }
            writeln!(core.stdout, "\t{}", file.name()).unwrap();
        }
    }
}
