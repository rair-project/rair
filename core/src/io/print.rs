//! commands handling raw data printing.

use crate::core::Core;
use crate::helper::{error_msg, expect, is_color, str_to_num, AddrMode};
use crate::writer::Writer;
use crate::Cmd;
use core::{cmp, fmt::Write as _};
use rair_env::Environment;
use std::io::Write;
use yansi::Paint;

#[derive(Default)]
pub struct PrintHex;

fn one_byte(_: &str, value: &str, _: &Environment<Core>, _: &mut Core) -> bool {
    value.len() == 1
}

impl PrintHex {
    pub fn new(core: &mut Core) -> Self {
        let env = core.env.clone();
        env.write()
            .add_str_with_cb(
                "printHex.headerColor",
                "color.6",
                "Color used in the header of `printHex` command",
                core,
                is_color,
            )
            .unwrap();
        env.write()
            .add_str_with_cb(
                "printHex.nonPrintColor",
                "color.5",
                "Color used in the Ascii section for non printable ASCII when using the `printHex` command",
                core,
                is_color,
            )
            .unwrap();
        env.write()
            .add_str_with_cb(
                "printHex.nonPrintReplace",
                ".",
                "Text used in the Ascii section to replace non printable ASCII when using the `printHex` command",
                core,
                one_byte,
            )
            .unwrap();
        env.write()
            .add_str_with_cb(
                "printHex.gapReplace",
                "#",
                "Text used to replace gaps when using the `printHex` command",
                core,
                one_byte,
            )
            .unwrap();

        Self
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
        let data_or_no_data = match core.mode {
            AddrMode::Phy => core.io.pread_sparce(loc, size),
            AddrMode::Vir => core.io.vread_sparce(loc, size),
        };
        let data = match data_or_no_data {
            Ok(d) => d,
            Err(e) => return error_msg(core, "Read Failed", &e.to_string()),
        };
        let env = core.env.read();
        let color = env.get_str("printHex.headerColor").unwrap();
        let banner = env.get_color(color).unwrap();
        let color = env.get_str("printHex.nonPrintColor").unwrap();
        let na = core.env.read().get_color(color).unwrap();
        let gap = env.get_str("printHex.gapReplace").unwrap();
        let no_print = env.get_str("printHex.nonPrintReplace").unwrap();

        writeln!(
            core.stdout,
            "{}",
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF"
                .rgb(banner.0, banner.1, banner.2,)
        )
        .unwrap();
        for i in (0..size).step_by(16) {
            write!(
                core.stdout,
                "{} ",
                format!("0x{:08x}", loc + i).rgb(banner.0, banner.1, banner.2)
            )
            .unwrap();
            let mut ascii = Writer::new_buf();
            let mut hex = Writer::new_buf();
            for j in i..cmp::min(i + 16, size) {
                if let Some(c) = data.get(&(j + loc)) {
                    if j % 2 == 0 {
                        write!(hex, "{c:02x}").unwrap();
                    } else {
                        write!(hex, "{c:02x} ").unwrap();
                    }
                    if *c >= 0x21 && *c <= 0x7E {
                        write!(ascii, "{}", *c as char).unwrap();
                    } else {
                        write!(ascii, "{}", no_print.rgb(na.0, na.1, na.2)).unwrap();
                    }
                } else {
                    if j % 2 == 0 {
                        write!(hex, "{gap}{gap}").unwrap();
                    } else {
                        write!(hex, "{gap}{gap} ").unwrap();
                    }
                    write!(ascii, "{}", gap.rgb(na.0, na.1, na.2)).unwrap();
                }
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

#[derive(Default)]
pub struct PrintBase;

impl PrintBase {
    pub fn new() -> Self {
        Self
    }
}
fn encode_bin(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 8);
    for byte in data {
        write!(out, "{byte:08b}").unwrap();
    }
    out
}
fn encode_hex(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for byte in data {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}
impl Cmd for PrintBase {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 2 {
            expect(core, args.len() as u64, 2);
            return;
        }
        let size = match str_to_num(&args[1]) {
            Ok(size) => size as usize,
            Err(e) => {
                let err_str = format!("{e}");
                error_msg(core, "Failed to parse size", &err_str);
                return;
            }
        };
        let mut data = vec![0; size];
        if size == 0 {
            return;
        }
        let loc = core.get_loc();
        let error = match core.mode {
            AddrMode::Phy => core.io.pread(loc, &mut data),
            AddrMode::Vir => core.io.vread(loc, &mut data),
        };
        if let Err(e) = error {
            error_msg(core, "Read Failed", &e.to_string());
            return;
        }
        let data_str = match args[0].as_ref() {
            "2" => encode_bin(&data),
            "16" => encode_hex(&data),
            _ => return error_msg(core, "Failed to print data", "Invalid base"),
        };
        writeln!(core.stdout, "{data_str}").unwrap();
    }
    fn commands(&self) -> &'static [&'static str] {
        &["printBase", "pb"]
    }

    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[(
            "[base] [size]",
            "Print data stream at current location in [base] format.  Supported bases: 2, 16.",
        )]
    }
}

#[derive(Default)]
pub struct PrintCSV;

impl PrintCSV {
    pub fn new() -> Self {
        Self
    }
}

fn csv8(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 6);
    let mut terminal;
    for (i, byte) in data.iter().enumerate() {
        if i == data.len() - 1 {
            terminal = "";
        } else if (i + 1) % 16 != 0 || i == 0 {
            terminal = ", ";
        } else {
            terminal = ",\n";
        }
        write!(out, "0x{byte:02x}{terminal}").unwrap();
    }
    out
}

fn csv16(data: &[u8]) -> String {
    // Data must be guaranteed to be of even length
    let mut out = String::with_capacity(data.len() * 4);
    let mut terminal;
    for i in (0..data.len()).step_by(2) {
        if i == data.len() - 2 {
            terminal = "";
        } else if (i + 2) % 24 != 0 || i == 0 {
            terminal = ", ";
        } else {
            terminal = ",\n";
        }
        write!(out, "0x{:02x}{:02x}{}", data[i + 1], data[i], terminal).unwrap();
    }
    out
}

fn csv32(data: &[u8]) -> String {
    // Data must be guaranteed to be of even length
    let mut out = String::with_capacity(data.len() * 3);
    let mut terminal;
    for i in (0..data.len()).step_by(4) {
        if i == data.len() - 4 {
            terminal = "";
        } else if (i + 4) % 32 != 0 || i == 0 {
            terminal = ", ";
        } else {
            terminal = ",\n";
        }
        out += "0x";
        for j in (0..4).rev() {
            write!(out, "{:02x}", data[i + j]).unwrap();
        }
        out += terminal;
    }
    out
}

fn csv64(data: &[u8]) -> String {
    // Data must be guaranteed to be of even length
    let mut out = String::with_capacity(data.len() * 3);
    let mut terminal;
    for i in (0..data.len()).step_by(8) {
        if i == data.len() - 8 {
            terminal = "";
        } else if (i + 8) % 32 != 0 || i == 0 {
            terminal = ", ";
        } else {
            terminal = ",\n";
        }
        out += "0x";
        for j in (0..8).rev() {
            write!(out, "{:02x}", data[i + j]).unwrap();
        }
        out += terminal;
    }
    out
}

fn csv128(data: &[u8]) -> String {
    // Data must be guaranteed to be of even length
    let mut out = String::with_capacity(data.len() * 3);
    let mut terminal;
    for i in (0..data.len()).step_by(16) {
        if i == data.len() - 16 {
            terminal = "";
        } else if (i + 16) % 32 != 0 || i == 0 {
            terminal = ", ";
        } else {
            terminal = ",\n";
        }
        out += "0x";
        for j in (0..16).rev() {
            write!(out, "{:02x}", data[i + j]).unwrap();
        }
        out += terminal;
    }
    out
}

fn csv256(data: &[u8]) -> String {
    // Data must be guaranteed to be of even length
    let mut out = String::with_capacity(data.len() * 3);
    let mut terminal;
    for i in (0..data.len()).step_by(32) {
        if i == data.len() - 32 {
            terminal = "";
        } else if (i + 32) % 64 != 0 || i == 0 {
            terminal = ", ";
        } else {
            terminal = ",\n";
        }
        out += "0x";
        for j in (0..32).rev() {
            write!(out, "{:02x}", data[i + j]).unwrap();
        }
        out += terminal;
    }
    out
}

fn csv512(data: &[u8]) -> String {
    // Data must be guaranteed to be of even length
    let mut out = String::with_capacity(data.len() * 3);
    let mut terminal;
    for i in (0..data.len()).step_by(64) {
        if i == data.len() - 64 {
            terminal = "";
        } else {
            terminal = ",\n";
        }
        out += "0x";
        for j in (0..64).rev() {
            write!(out, "{:02x}", data[i + j]).unwrap();
        }
        out += terminal;
    }
    out
}

impl Cmd for PrintCSV {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 2 {
            expect(core, args.len() as u64, 2);
            return;
        }
        let count = match str_to_num(&args[1]) {
            Ok(count) => count as usize,
            Err(e) => {
                let err_str = format!("{e}");
                error_msg(core, "Failed to parse count", &err_str);
                return;
            }
        };
        let bsize = match str_to_num(&args[0]) {
            Ok(size) => size as usize,
            Err(e) => {
                let err_str = format!("{e}");
                error_msg(core, "Failed to parse size", &err_str);
                return;
            }
        };
        let size = bsize / 8 * count;
        if count == 0 {
            return;
        }
        if size == 0 {
            return error_msg(core, "Failed to print data", "Invalid size");
        }
        let mut data = vec![0; size];

        let loc = core.get_loc();
        let error = match core.mode {
            AddrMode::Phy => core.io.pread(loc, &mut data),
            AddrMode::Vir => core.io.vread(loc, &mut data),
        };
        if let Err(e) = error {
            error_msg(core, "Read Failed", &e.to_string());
            return;
        }
        let data_str = match bsize {
            8 => csv8(&data),
            16 => csv16(&data),
            32 => csv32(&data),
            64 => csv64(&data),
            128 => csv128(&data),
            256 => csv256(&data),
            512 => csv512(&data),
            _ => return error_msg(core, "Failed to print data", "Invalid size"),
        };
        writeln!(core.stdout, "{data_str}").unwrap();
    }
    fn commands(&self) -> &'static [&'static str] {
        &["printCSV", "pcsv"]
    }
    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[(
            "[size] [count]",
            concat!(
                "Print data at current location as unsigned ",
                "comma seperated values, each value of size [size] bits.  ",
                "Supported size: 8, 16, 32, 64, 128, 256, 512."
            ),
        )]
    }
}

#[derive(Default)]
pub struct PrintSignedCSV;

impl PrintSignedCSV {
    pub fn new() -> Self {
        Self
    }
}

fn scsv8(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 6);
    let mut terminal;
    for (i, byte) in data.iter().enumerate() {
        if i == data.len() - 1 {
            terminal = "";
        } else if (i + 1) % 16 != 0 || i == 0 {
            terminal = ", ";
        } else {
            terminal = ",\n";
        }
        write!(out, "{}{}", *byte as i8, terminal).unwrap();
    }
    out
}

fn scsv16(data: &[u8]) -> String {
    // Data must be guaranteed to be of even length
    let mut out = String::with_capacity(data.len() * 4);
    let mut terminal;
    for i in (0..data.len()).step_by(2) {
        if i == data.len() - 2 {
            terminal = "";
        } else if (i + 2) % 24 != 0 || i == 0 {
            terminal = ", ";
        } else {
            terminal = ",\n";
        }
        let x = ((data[i + 1] as u16) << 8i32) + data[i] as u16;
        write!(out, "{}{}", x as i16, terminal).unwrap();
    }
    out
}

fn scsv32(data: &[u8]) -> String {
    // Data must be guaranteed to be of even length
    let mut out = String::with_capacity(data.len() * 3);
    let mut terminal;
    for i in (0..data.len()).step_by(4) {
        if i == data.len() - 4 {
            terminal = "";
        } else if (i + 4) % 32 != 0 || i == 0 {
            terminal = ", ";
        } else {
            terminal = ",\n";
        }
        let mut x = 0u32;
        for j in (0..4).rev() {
            x = (x << 8i32) + data[i + j] as u32;
        }
        write!(out, "{}{}", x as i32, terminal).unwrap();
    }
    out
}

fn scsv64(data: &[u8]) -> String {
    // Data must be guaranteed to be of even length
    let mut out = String::with_capacity(data.len() * 3);
    let mut terminal;
    for i in (0..data.len()).step_by(8) {
        if i == data.len() - 8 {
            terminal = "";
        } else if (i + 8) % 32 != 0 || i == 0 {
            terminal = ", ";
        } else {
            terminal = ",\n";
        }
        let mut x = 0u64;
        for j in (0..8).rev() {
            x = (x << 8i32) + data[i + j] as u64;
        }
        write!(out, "{}{}", x as i64, terminal).unwrap();
    }
    out
}

fn scsv128(data: &[u8]) -> String {
    // Data must be guaranteed to be of even length
    let mut out = String::with_capacity(data.len() * 3);
    let mut terminal;
    for i in (0..data.len()).step_by(16) {
        if i == data.len() - 16 {
            terminal = "";
        } else if (i + 16) % 32 != 0 || i == 0 {
            terminal = ", ";
        } else {
            terminal = ",\n";
        }
        let mut x = 0u128;
        for j in (0..16).rev() {
            x = (x << 8i32) + data[i + j] as u128;
        }
        write!(out, "{}{}", x as i128, terminal).unwrap();
    }
    out
}

impl Cmd for PrintSignedCSV {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 2 {
            expect(core, args.len() as u64, 2);
            return;
        }
        let count = match str_to_num(&args[1]) {
            Ok(count) => count as usize,
            Err(e) => {
                let err_str = format!("{e}");
                error_msg(core, "Failed to parse count", &err_str);
                return;
            }
        };
        let bsize = match str_to_num(&args[0]) {
            Ok(size) => size as usize,
            Err(e) => {
                let err_str = format!("{e}");
                error_msg(core, "Failed to parse size", &err_str);
                return;
            }
        };
        let size = bsize / 8 * count;
        if count == 0 {
            return;
        }
        if size == 0 {
            return error_msg(core, "Failed to print data", "Invalid size");
        }
        let mut data = vec![0; size];

        let loc = core.get_loc();
        let error = match core.mode {
            AddrMode::Phy => core.io.pread(loc, &mut data),
            AddrMode::Vir => core.io.vread(loc, &mut data),
        };
        if let Err(e) = error {
            error_msg(core, "Read Failed", &e.to_string());
            return;
        }
        let data_str = match bsize {
            8 => scsv8(&data),
            16 => scsv16(&data),
            32 => scsv32(&data),
            64 => scsv64(&data),
            128 => scsv128(&data),
            _ => return error_msg(core, "Failed to print data", "Invalid size"),
        };
        writeln!(core.stdout, "{data_str}").unwrap();
    }
    fn commands(&self) -> &'static [&'static str] {
        &["printSCSV", "pscsv"]
    }

    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[(
            "[size] [count]",
            concat!(
                "Print data at current location as signed comma ",
                "seperated values, each value of size [size] bits.  ",
                "Supported size: 8, 16, 32, 64, 128."
            ),
        )]
    }
}

#[cfg(test)]
mod test_print_hex {
    use super::*;
    use crate::{writer::Writer, CmdOps};
    use rair_io::*;
    use std::path::Path;
    use test_file::*;

    #[test]
    fn test_help() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let pb = PrintBase::new();
        let pcsv = PrintCSV::new();
        let pscsv = PrintSignedCSV::new();
        core.help("px");
        pb.help(&mut core);
        pcsv.help(&mut core);
        pscsv.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [printHex | px]\n\
             Usage:\n\
             px [size]\tView data at current location in hex format.\n\
             Commands: [printBase | pb]\n\
             Usage:\n\
             pb [base] [size]\tPrint data stream at current location in [base] format.  Supported bases: 2, 16.\n\
             Commands: [printCSV | pcsv]\n\
             Usage:\n\
             pcsv [size] [count]\tPrint data at current location as unsigned comma seperated values, each value of size [size] bits.  Supported size: 8, 16, 32, 64, 128, 256, 512.\n\
             Commands: [printSCSV | pscsv]\n\
             Usage:\n\
             pscsv [size] [count]\tPrint data at current location as signed comma seperated values, each value of size [size] bits.  Supported size: 8, 16, 32, 64, 128.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    fn test_px_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run("px", &["0x0".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x1".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 00                                       .\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x2".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001                                     ..\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x3".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 01                                  ...\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x4".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102                                ....\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x5".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 03                             .....\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x6".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305                           ......\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x7".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 08                        .......\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x8".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d                      ........\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x9".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 15                   .........\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0xa".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522                 .........\"\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0xb".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 37              .........\"7\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0xc".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759            .........\"7Y\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0xd".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90         .........\"7Y.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0xe".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9       .........\"7Y..\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0xf".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 79    .........\"7Y..y\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x10".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x11".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db                                       .\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x12".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d                                     .=\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x13".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 18                                  .=.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x14".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855                                .=.U\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x15".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6d                             .=.Um\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x16".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2                           .=.Um.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x17".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2f                        .=.Um./\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x18".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1                      .=.Um./.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x19".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 20                   .=.Um./..\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x1a".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011                 .=.Um./...\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x1b".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 31              .=.Um./...1\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x1c".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142            .=.Um./...1B\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x1d".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73         .=.Um./...1Bs\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x1e".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5       .=.Um./...1Bs.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x1f".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28    .=.Um./...1Bs.(\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x20".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x21".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n\
             0x00000020 05                                       .\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x22".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n\
             0x00000020 05e2                                     ..\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x23".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n\
             0x00000020 05e2 e7                                  ...\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_px() {
        operate_on_file(&test_px_cb, DATA);
    }

    fn test_px_vir_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run("px", &["0x0".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.io.map(0x0, 0x500, 0x10).unwrap();
        core.io.map(0x15, 0x515, 0x5).unwrap();
        core.io.map(0x10, 0x520, 0x20).unwrap();
        core.io.map(0x20, 0x540, 0x20).unwrap();
        core.set_loc(0x500);
        core.run("px", &["0x65".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000500 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000510 #### #### ##c2 2ff1 2011 #### #### ####  #####./...######\n\
             0x00000520 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n\
             0x00000530 05e2 e7c9 b079 29a2 cb6d 38a5 dd82 5fe1  .....y)..m8..._.\n\
             0x00000540 05e2 e7c9 b079 29a2 cb6d 38a5 dd82 5fe1  .....y)..m8..._.\n\
             0x00000550 4021 6182 e365 48ad f5a2 9739 d009 d9e2  @!a..eH....9....\n\
             0x00000560 #### #### ##                             #####\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_px_vir() {
        operate_on_file(&test_px_vir_cb, DATA);
    }

    #[test]
    fn test_px_err() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.run("px", &[]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 1 argument(s), found 0.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        core.run("px", &["0x".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: cannot parse integer from empty string\nExpect Hex, binary, Octal or Decimal value but found 0x instead.\n"
        );
    }

    #[test]
    fn test_pb_2() {
        let mut core = Core::new_no_colors();
        let mut pb = PrintBase::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/base64/no_padding.b64",
                IoMode::READ,
            )
            .unwrap();
        pb.run(&mut core, &["2".to_owned(), "16".to_owned()]);
        core.io.map(0, 0x500, 16).unwrap();
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "01010110010001110110100001101100\
             01001001010010000100011000110001\
             01100001010101110100111001110010\
             01001001010001110100101001111001\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pb.run(&mut core, &["2".to_owned(), "16".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "01010110010001110110100001101100\
             01001001010010000100011000110001\
             01100001010101110100111001110010\
             01001001010001110100101001111001\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_pb_16() {
        let mut core = Core::new_no_colors();
        let mut pb = PrintBase::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/base64/no_padding.b64",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 16).unwrap();
        pb.run(&mut core, &["16".to_owned(), "16".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "5647686c4948463161574e7249474a79\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pb.run(&mut core, &["16".to_owned(), "16".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "5647686c4948463161574e7249474a79\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_pb_error() {
        let mut core = Core::new_no_colors();
        let mut pb = PrintBase::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/base64/no_padding.b64",
                IoMode::READ,
            )
            .unwrap();
        pb.run(&mut core, &["16".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 2 argument(s), found 1.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        pb.run(&mut core, &["16".to_owned(), "x".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse size\ninvalid digit found in string\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        pb.run(&mut core, &["16".to_owned(), "0x5000".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Read Failed\nCannot resolve address.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        pb.run(&mut core, &["5".to_owned(), "5".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to print data\nInvalid base\n"
        );
    }
    #[test]
    fn test_pcsv_8() {
        let mut core = Core::new_no_colors();
        let mut pcsv = PrintCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/base64/no_padding.b64",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 35).unwrap();
        pcsv.run(&mut core, &["8".to_owned(), "35".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x56, 0x47, 0x68, 0x6c, 0x49, 0x48, 0x46, 0x31, 0x61, 0x57, 0x4e, 0x72, 0x49, 0x47, 0x4a, 0x79,\n\
             0x62, 0x33, 0x64, 0x75, 0x49, 0x47, 0x5a, 0x76, 0x65, 0x43, 0x42, 0x71, 0x64, 0x57, 0x31, 0x77,\n\
             0x5a, 0x57, 0x51\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pcsv.run(&mut core, &["8".to_owned(), "35".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x56, 0x47, 0x68, 0x6c, 0x49, 0x48, 0x46, 0x31, 0x61, 0x57, 0x4e, 0x72, 0x49, 0x47, 0x4a, 0x79,\n\
             0x62, 0x33, 0x64, 0x75, 0x49, 0x47, 0x5a, 0x76, 0x65, 0x43, 0x42, 0x71, 0x64, 0x57, 0x31, 0x77,\n\
             0x5a, 0x57, 0x51\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_pcsv_16() {
        let mut core = Core::new_no_colors();
        let mut pcsv = PrintCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/base64/no_padding.b64",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 52).unwrap();
        pcsv.run(&mut core, &["16".to_owned(), "26".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x4756, 0x6c68, 0x4849, 0x3146, 0x5761, 0x724e, 0x4749, 0x794a, 0x3362, 0x7564, 0x4749, 0x765a,\n\
             0x4365, 0x7142, 0x5764, 0x7731, 0x575a, 0x6751, 0x3362, 0x6c5a, 0x6963, 0x3042, 0x4761, 0x6755,\n\
             0x4762, 0x3646\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pcsv.run(&mut core, &["16".to_owned(), "26".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x4756, 0x6c68, 0x4849, 0x3146, 0x5761, 0x724e, 0x4749, 0x794a, 0x3362, 0x7564, 0x4749, 0x765a,\n\
             0x4365, 0x7142, 0x5764, 0x7731, 0x575a, 0x6751, 0x3362, 0x6c5a, 0x6963, 0x3042, 0x4761, 0x6755,\n\
             0x4762, 0x3646\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_pcsv_32() {
        let mut core = Core::new_no_colors();
        let mut pcsv = PrintCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/base64/no_padding.b64",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 60).unwrap();
        pcsv.run(&mut core, &["32".to_owned(), "15".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x6c684756, 0x31464849, 0x724e5761, 0x794a4749, 0x75643362, 0x765a4749, 0x71424365, 0x77315764,\n\
             0x6751575a, 0x6c5a3362, 0x30426963, 0x67554761, 0x36464762, 0x6b425365, 0x75633262\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pcsv.run(&mut core, &["32".to_owned(), "15".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x6c684756, 0x31464849, 0x724e5761, 0x794a4749, 0x75643362, 0x765a4749, 0x71424365, 0x77315764,\n\
             0x6751575a, 0x6c5a3362, 0x30426963, 0x67554761, 0x36464762, 0x6b425365, 0x75633262\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_pcsv_64() {
        let mut core = Core::new_no_colors();
        let mut pcsv = PrintCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/srec/record_0_1_9.srec",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 700).unwrap();
        pcsv.run(&mut core, &["64".to_owned(), "15".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x3030303031323053, 0x3035423438333633, 0x3032373446343235, 0x3033323330323032,\n\
             0x3134353432353334, 0x3032343435343435, 0x3534303239353234, 0x3633393533353134,\n\
             0x0a0d443642343833, 0x3030303133323153, 0x3030303039373234, 0x4333303132343131,\n\
             0x4333323130323030, 0x4634453430303030, 0x3130303043333231\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pcsv.run(&mut core, &["64".to_owned(), "15".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x3030303031323053, 0x3035423438333633, 0x3032373446343235, 0x3033323330323032,\n\
             0x3134353432353334, 0x3032343435343435, 0x3534303239353234, 0x3633393533353134,\n\
             0x0a0d443642343833, 0x3030303133323153, 0x3030303039373234, 0x4333303132343131,\n\
             0x4333323130323030, 0x4634453430303030, 0x3130303043333231\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_pcsv_128() {
        let mut core = Core::new_no_colors();
        let mut pcsv = PrintCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/srec/record_0_1_9.srec",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 700).unwrap();
        pcsv.run(&mut core, &["128".to_owned(), "7".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x30354234383336333030303031323053, 0x30333233303230323032373446343235,\n\
             0x30323434353434353134353432353334, 0x36333935333531343534303239353234,\n\
             0x30303031333231530a0d443642343833, 0x43333031323431313030303039373234,\n\
             0x46344534303030304333323130323030\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pcsv.run(&mut core, &["128".to_owned(), "7".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x30354234383336333030303031323053, 0x30333233303230323032373446343235,\n\
             0x30323434353434353134353432353334, 0x36333935333531343534303239353234,\n\
             0x30303031333231530a0d443642343833, 0x43333031323431313030303039373234,\n\
             0x46344534303030304333323130323030\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_pcsv_256() {
        let mut core = Core::new_no_colors();
        let mut pcsv = PrintCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/srec/record_0_1_9.srec",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 700).unwrap();
        pcsv.run(&mut core, &["256".to_owned(), "5".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x3033323330323032303237344634323530354234383336333030303031323053, 0x3633393533353134353430323935323430323434353434353134353432353334,\n\
             0x4333303132343131303030303937323430303031333231530a0d443642343833, 0x3134383246344534313030304333323146344534303030304333323130323030,\n\
             0x4334343230323031313231530a0d454231343632463445343230303043333231\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pcsv.run(&mut core, &["256".to_owned(), "5".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x3033323330323032303237344634323530354234383336333030303031323053, 0x3633393533353134353430323935323430323434353434353134353432353334,\n\
             0x4333303132343131303030303937323430303031333231530a0d443642343833, 0x3134383246344534313030304333323146344534303030304333323130323030,\n\
             0x4334343230323031313231530a0d454231343632463445343230303043333231\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_pcsv_512() {
        let mut core = Core::new_no_colors();
        let mut pcsv = PrintCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/srec/record_0_1_9.srec",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 700).unwrap();
        pcsv.run(&mut core, &["512".to_owned(), "3".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x36333935333531343534303239353234303234343534343531343534323533343033323330323032303237344634323530354234383336333030303031323053,\n\
             0x31343832463445343130303043333231463445343030303043333231303230304333303132343131303030303937323430303031333231530a0d443642343833,\n\
             0x31383430363035363030424531343030434232424634453438303030433330314334343230323031313231530a0d454231343632463445343230303043333231\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pcsv.run(&mut core, &["512".to_owned(), "3".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "0x36333935333531343534303239353234303234343534343531343534323533343033323330323032303237344634323530354234383336333030303031323053,\n\
             0x31343832463445343130303043333231463445343030303043333231303230304333303132343131303030303937323430303031333231530a0d443642343833,\n\
             0x31383430363035363030424531343030434232424634453438303030433330314334343230323031313231530a0d454231343632463445343230303043333231\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_pcsv_errors() {
        let mut core = Core::new_no_colors();
        let mut pcsv = PrintCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/srec/record_0_1_9.srec",
                IoMode::READ,
            )
            .unwrap();
        pcsv.run(&mut core, &["512".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 2 argument(s), found 1.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        pcsv.run(&mut core, &["512".to_owned(), "50x".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse count\ninvalid digit found in string\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        pcsv.run(&mut core, &["51x".to_owned(), "50".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse size\ninvalid digit found in string\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        pcsv.run(&mut core, &["51".to_owned(), "50".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to print data\nInvalid size\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        pcsv.run(&mut core, &["512".to_owned(), "500000".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Read Failed\nCannot resolve address.\n"
        );
    }

    #[test]
    fn test_pscsv_errors() {
        let mut core = Core::new_no_colors();
        let mut pscsv = PrintSignedCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/srec/record_0_1_9.srec",
                IoMode::READ,
            )
            .unwrap();
        pscsv.run(&mut core, &["128".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 2 argument(s), found 1.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        pscsv.run(&mut core, &["128".to_owned(), "50x".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse count\ninvalid digit found in string\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        pscsv.run(&mut core, &["12x".to_owned(), "50".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse size\ninvalid digit found in string\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        pscsv.run(&mut core, &["12".to_owned(), "50".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to print data\nInvalid size\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        pscsv.run(&mut core, &["128".to_owned(), "500000".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Read Failed\nCannot resolve address.\n"
        );
    }

    #[test]
    fn test_pscsv_8() {
        let mut core = Core::new_no_colors();
        let mut pscsv = PrintSignedCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/base64/no_padding.b64",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 35).unwrap();
        pscsv.run(&mut core, &["8".to_owned(), "35".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "86, 71, 104, 108, 73, 72, 70, 49, 97, 87, 78, 114, 73, 71, 74, 121,\n\
             98, 51, 100, 117, 73, 71, 90, 118, 101, 67, 66, 113, 100, 87, 49, 119,\n\
             90, 87, 81\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pscsv.run(&mut core, &["8".to_owned(), "35".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "86, 71, 104, 108, 73, 72, 70, 49, 97, 87, 78, 114, 73, 71, 74, 121,\n\
             98, 51, 100, 117, 73, 71, 90, 118, 101, 67, 66, 113, 100, 87, 49, 119,\n\
             90, 87, 81\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_pscsv_16() {
        let mut core = Core::new_no_colors();
        let mut pscsv = PrintSignedCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/base64/no_padding.b64",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 52).unwrap();
        pscsv.run(&mut core, &["16".to_owned(), "26".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "18262, 27752, 18505, 12614, 22369, 29262, 18249, 31050, 13154, 30052, 18249, 30298,\n\
             17253, 28994, 22372, 30513, 22362, 26449, 13154, 27738, 26979, 12354, 18273, 26453,\n\
             18274, 13894\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pscsv.run(&mut core, &["16".to_owned(), "26".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "18262, 27752, 18505, 12614, 22369, 29262, 18249, 31050, 13154, 30052, 18249, 30298,\n\
             17253, 28994, 22372, 30513, 22362, 26449, 13154, 27738, 26979, 12354, 18273, 26453,\n\
             18274, 13894\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_pscsv_32() {
        let mut core = Core::new_no_colors();
        let mut pscsv = PrintSignedCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/base64/no_padding.b64",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 60).unwrap();
        pscsv.run(&mut core, &["32".to_owned(), "15".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "1818773334, 826689609, 1917736801, 2034911049, 1969501026, 1985627977, 1900168037, 1999722340,\n\
             1733384026, 1817850722, 809658723, 1733642081, 910575458, 1799508837, 1969435234\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pscsv.run(&mut core, &["32".to_owned(), "15".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "1818773334, 826689609, 1917736801, 2034911049, 1969501026, 1985627977, 1900168037, 1999722340,\n\
             1733384026, 1817850722, 809658723, 1733642081, 910575458, 1799508837, 1969435234\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_pscsv_64() {
        let mut core = Core::new_no_colors();
        let mut pscsv = PrintSignedCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/srec/record_0_1_9.srec",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 700).unwrap();
        pscsv.run(&mut core, &["64".to_owned(), "15".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "3472328296244588627, 3473755479634818611, 3472898960311726645, 3473174933066100786,\n\
             3545517304944341812, 3472895661491631157, 3833742175065420340, 3905528202515525940,\n\
             724310114906159155, 3472328300573110611, 3472328296379134516, 4842267012207227185,\n\
             4842269211196796976, 5058744371892990000, 3544385890584572465\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pscsv.run(&mut core, &["64".to_owned(), "15".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "3472328296244588627, 3473755479634818611, 3472898960311726645, 3473174933066100786,\n\
             3545517304944341812, 3472895661491631157, 3833742175065420340, 3905528202515525940,\n\
             724310114906159155, 3472328300573110611, 3472328296379134516, 4842267012207227185,\n\
             4842269211196796976, 5058744371892990000, 3544385890584572465\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_pscsv_128() {
        let mut core = Core::new_no_colors();
        let mut pscsv = PrintSignedCSV::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io
            .open(
                "../testing_binaries/rio/srec/record_0_1_9.srec",
                IoMode::READ,
            )
            .unwrap();
        core.io.map(0, 0x500, 700).unwrap();
        pscsv.run(&mut core, &["128".to_owned(), "7".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "64079478307469671234530411534546514003, 64068769113493663281246783985836896821,\n\
             64063617462232360116848175623869641524, 72044279224458795675434962490882339380,\n\
             64053151500570986824316153078087956531, 89324060310752925109750682577375015476,\n\
             93317362762628561321598115244768636976\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.set_loc(0x500);
        pscsv.run(&mut core, &["128".to_owned(), "7".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "64079478307469671234530411534546514003, 64068769113493663281246783985836896821,\n\
             64063617462232360116848175623869641524, 72044279224458795675434962490882339380,\n\
             64053151500570986824316153078087956531, 89324060310752925109750682577375015476,\n\
             93317362762628561321598115244768636976\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
}
