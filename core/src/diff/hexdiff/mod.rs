use crate::{error_msg, expect_range, hex::HexWithoutEnv, str_to_num, Cmd, Core, Writer};
use core::cmp::min;
use std::io::Write;

pub struct HexDiff {
    inner: HexWithoutEnv,
}

impl HexDiff {
    pub fn new(core: &mut Core) -> Self {
        Self {
            inner: HexWithoutEnv::new(core),
        }
    }
    fn parse_args(core: &mut Core, args: &[String]) -> Option<(u64, u64, u64)> {
        let args: Vec<_> = args.iter().map(|s| str_to_num(s)).collect();
        if args.len() == 2 {
            let addr1 = core.get_loc();
            let addr2 = match &args[0] {
                Ok(addr) => *addr,
                Err(e) => {
                    let err_str = format!("{e}");
                    error_msg(core, "Failed to parse addr", &err_str);
                    return None;
                }
            };
            let size = match &args[1] {
                Ok(size) => *size,
                Err(e) => {
                    let err_str = format!("{e}");
                    error_msg(core, "Failed to parse size", &err_str);
                    return None;
                }
            };
            Some((addr1, addr2, size))
        } else {
            let addr1 = match &args[0] {
                Ok(addr) => *addr,
                Err(e) => {
                    let err_str = format!("{e}");
                    error_msg(core, "Failed to parse addr1", &err_str);
                    return None;
                }
            };
            let addr2 = match &args[1] {
                Ok(addr) => *addr,
                Err(e) => {
                    let err_str = format!("{e}");
                    error_msg(core, "Failed to parse addr2", &err_str);
                    return None;
                }
            };
            let size = match &args[2] {
                Ok(size) => *size,
                Err(e) => {
                    let err_str = format!("{e}");
                    error_msg(core, "Failed to parse size", &err_str);
                    return None;
                }
            };
            Some((addr1, addr2, size))
        }
    }

    //print enough spaces to padd
    pub fn hex_space(counter: usize) -> String {
        let mut s = " ".to_owned();
        for i in counter..16 {
            s.push_str("  ");
            if i % 2 != 0 {
                s.push(' ');
            }
        }
        s
    }

    pub fn ascii_space(counter: usize) -> String {
        let mut s = String::new();
        for _ in counter..16 {
            s.push(' ');
        }
        s
    }
}

impl Cmd for HexDiff {
    fn commands(&self) -> &'static [&'static str] {
        &["hd", "hexDiff"]
    }

    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[
            (
                "[addr] [size]",
                "\tPrint binary diff between current location and [addr] for [size] bytes.",
            ),
            (
                "[addr1] [addr2] [size]",
                "Print binary diff between [addr1] and [addr2] for [size] bytes.",
            ),
        ]
    }

    fn run(&mut self, core: &mut Core, args: &[String]) {
        if !(2..4).contains(&args.len()) {
            expect_range(core, args.len() as u64, 2, 3);
            return;
        }
        let Some((addr1, addr2, size)) = Self::parse_args(core, args) else {
            return;
        };
        if size == 0 {
            return;
        }
        let data1 = match core.read_sparce(addr1, size) {
            Ok(d) => d,
            Err(e) => return error_msg(core, "Read Failed", &e.to_string()),
        };
        let data2 = match core.read_sparce(addr2, size) {
            Ok(d) => d,
            Err(e) => return error_msg(core, "Read Failed", &e.to_string()),
        };
        let env = self.inner.get_env(core);
        env.print_double_banner(&mut core.stdout);
        for i in (0..size).step_by(16) {
            let mut ascii1 = Writer::new_buf();
            let mut hex1 = Writer::new_buf();
            let mut ascii2 = Writer::new_buf();
            let mut hex2 = Writer::new_buf();

            for j in i..min(i + 16, size) {
                let byte1 = data1.get(&(j + addr1)).copied();
                let byte2 = data2.get(&(j + addr2)).copied();
                env.print_hex_with_highlight(byte1, &mut hex1, j % 2 != 0, byte1 != byte2);
                env.print_ascii_with_highlight(byte1, &mut ascii1, byte1 != byte2);
                env.print_hex_with_highlight(byte2, &mut hex2, j % 2 != 0, byte1 != byte2);
                env.print_ascii_with_highlight(byte2, &mut ascii2, byte1 != byte2);
            }
            env.print_addr(&mut core.stdout, addr1);
            let hex_space = if i + 16 < size || size % 16 == 0 {
                " ".to_owned()
            } else {
                Self::hex_space(size as usize % 16)
            };
            let ascii_space = if i + 16 < size || size % 16 == 0 {
                String::new()
            } else {
                Self::ascii_space(size as usize % 16)
            };
            write!(
                core.stdout,
                "{}{hex_space}{}{ascii_space}",
                hex1.utf8_string().unwrap(),
                ascii1.utf8_string().unwrap(),
            )
            .unwrap();
            env.print_separator(&mut core.stdout);
            env.print_addr(&mut core.stdout, addr2);
            writeln!(
                core.stdout,
                "{}{hex_space}{}",
                hex2.utf8_string().unwrap(),
                ascii2.utf8_string().unwrap(),
            )
            .unwrap();
        }
    }
}

#[cfg(test)]
mod test;
