use crate::{expect, is_color, Cmd, Core};
use std::io::Write;
use yansi::Paint;

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
    fn commands(&self) -> &'static [&'static str] {
        &["maps"]
    }
    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[("", "List all memory maps.")]
    }
}
