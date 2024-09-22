use crate::{error_msg, expect, hex::HexWithoutEnv, str_to_num, Cmd, Core, Writer};
use core::cmp;
use std::io::Write;
use yansi::Paint;

pub struct PrintHex {
    inner: HexWithoutEnv,
}

impl PrintHex {
    pub fn new(core: &mut Core) -> Self {
        Self {
            inner: HexWithoutEnv::new(core),
        }
    }
}

impl Cmd for PrintHex {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        // we can always optimize by try and using pread an vread.
        // If they fail, only then we might want to attempt the sparce version.
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return;
        }
        let size = match str_to_num(&args[0]) {
            Ok(s) => s,
            Err(e) => {
                return error_msg(
                    core,
                    &e.to_string(),
                    &format!(
                        "Expect Hex, binary, Octal or Decimal value but found {} instead.",
                        &args[0].primary().italic()
                    ),
                )
            }
        };
        if size == 0 {
            return;
        }
        let loc = core.get_loc();
        let data = match core.read_sparce(loc, size) {
            Ok(d) => d,
            Err(e) => return error_msg(core, "Read Failed", &e.to_string()),
        };
        let env = self.inner.get_env(core);
        env.print_banner(&mut core.stdout);
        for i in (0..size).step_by(16) {
            env.print_addr(&mut core.stdout, loc + i);
            let mut ascii = Writer::new_buf();
            let mut hex = Writer::new_buf();
            for j in i..cmp::min(i + 16, size) {
                let byte = data.get(&(j + loc)).copied();
                env.print_hex(byte, &mut hex, j % 2 != 0);
                env.print_ascii(byte, &mut ascii);
            }
            writeln!(
                core.stdout,
                "{: <40} {}",
                hex.utf8_string().unwrap(),
                ascii.utf8_string().unwrap()
            )
            .unwrap();
        }
    }

    fn commands(&self) -> &'static [&'static str] {
        &["printHex", "px"]
    }

    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[("[size]", "View data at current location in hex format.")]
    }
}
