/*
 * print: commands handling raw data printing.
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
use std::cmp;
use std::io::Write;
use writer::*;
use yansi::Paint;

#[derive(Default)]
pub struct PrintHex {}

impl PrintHex {
    pub fn new() -> Self {
        Default::default()
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
                    &format!("Expect Hex, binary, Octal or Decimal value but found {} instead.", Paint::default(&args[0]).italic()),
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
        let banner = core.color_palette[5];
        let na = core.color_palette[4];
        writeln!(
            core.stdout,
            "{}",
            Paint::rgb(banner.0, banner.1, banner.2, "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF")
        )
        .unwrap();
        for i in (0..size).step_by(16) {
            write!(core.stdout, "{} ", Paint::rgb(banner.0, banner.1, banner.2, format!("0x{:08x}", loc + i))).unwrap();
            let mut ascii = Writer::new_buf();
            let mut hex = Writer::new_buf();
            for j in i..cmp::min(i + 16, size) {
                if let Some(c) = data.get(&(j + loc)) {
                    if j % 2 == 0 {
                        write!(hex, "{:02x}", c).unwrap();
                    } else {
                        write!(hex, "{:02x} ", c).unwrap();
                    }
                    if *c >= 0x21 && *c <= 0x7E {
                        write!(ascii, "{}", *c as char).unwrap()
                    } else {
                        write!(ascii, "{}", Paint::rgb(na.0, na.1, na.2, ".")).unwrap();
                    }
                } else {
                    if j % 2 == 0 {
                        write!(hex, "##").unwrap();
                    } else {
                        write!(hex, "## ").unwrap();
                    }
                    write!(ascii, "{}", Paint::rgb(na.0, na.1, na.2, "#")).unwrap();
                }
            }
            writeln!(core.stdout, "{: <40} {}", hex.utf8_string().unwrap(), ascii.utf8_string().unwrap()).unwrap();
        }
    }
    fn help(&self, core: &mut Core) {
        help(core, &"printHex", &"px", vec![("[size]", "View data at current location in hex format.")]);
    }
}

#[derive(Default)]
pub struct PrintBase {}

impl PrintBase {
    pub fn new() -> Self {
        Default::default()
    }
}
fn encode_bin(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 8);
    for byte in data {
        out += &format!("{:08b}", byte);
    }
    return out;
}
fn encode_hex(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for byte in data {
        out += &format!("{:02x}", byte);
    }
    return out;
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
                let err_str = format!("{}", e);
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
        writeln!(core.stdout, "{}", data_str).unwrap();
    }
    fn help(&self, core: &mut Core) {
        help(core, &"printBase", &"pb", vec![("[base] [size]", "Print data stream at current location in [base] format.")]);
        writeln!(core.stdout, "Supported bases: 2, 16").unwrap();
    }
}

#[derive(Default)]
pub struct PrintCSV {}

impl PrintCSV {
    pub fn new() -> Self {
        Default::default()
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
        out += &format!("0x{:02x}{}", byte, terminal);
    }
    return out;
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
        out += &format!("0x{:02x}{:02x}{}", data[i + 1], data[i], terminal);
    }
    return out;
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
            out += &format!("{:02x}", data[i + j]);
        }
        out += terminal
    }
    return out;
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
            out += &format!("{:02x}", data[i + j]);
        }
        out += terminal
    }
    return out;
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
            out += &format!("{:02x}", data[i + j]);
        }
        out += terminal
    }
    return out;
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
            out += &format!("{:02x}", data[i + j]);
        }
        out += terminal
    }
    return out;
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
            out += &format!("{:02x}", data[i + j]);
        }
        out += terminal
    }
    return out;
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
                let err_str = format!("{}", e);
                error_msg(core, "Failed to parse count", &err_str);
                return;
            }
        };
        let bsize = match str_to_num(&args[0]) {
            Ok(size) => size as usize,
            Err(e) => {
                let err_str = format!("{}", e);
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
        writeln!(core.stdout, "{}", data_str).unwrap();
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            &"printCSV",
            &"pcsv",
            vec![("[size] [count]", "Print data at current location as unsigned comma seperated values, each value of size [size] bits.")],
        );
        writeln!(core.stdout, "Supported size: 8, 16, 32, 64, 128, 256, 512.").unwrap();
    }
}

#[derive(Default)]
pub struct PrintSignedCSV {}

impl PrintSignedCSV {
    pub fn new() -> Self {
        Default::default()
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
        out += &format!("{}{}", *byte as i8, terminal);
    }
    return out;
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
        let x = ((data[i + 1] as u16) << 8) + data[i] as u16;
        out += &format!("{}{}", x as i16, terminal);
    }
    return out;
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
            x = (x << 8) + data[i + j] as u32;
        }
        out += &format!("{}{}", x as i32, terminal);
    }
    return out;
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
            x = (x << 8) + data[i + j] as u64;
        }
        out += &format!("{}{}", x as i64, terminal);
    }
    return out;
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
            x = (x << 8) + data[i + j] as u128;
        }
        out += &format!("{}{}", x as i128, terminal);
    }
    return out;
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
                let err_str = format!("{}", e);
                error_msg(core, "Failed to parse count", &err_str);
                return;
            }
        };
        let bsize = match str_to_num(&args[0]) {
            Ok(size) => size as usize,
            Err(e) => {
                let err_str = format!("{}", e);
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
        writeln!(core.stdout, "{}", data_str).unwrap();
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            &"printSCSV",
            &"pscsv",
            vec![("[size] [count]", "Print data at current location as signed comma seperated values, each value of size [size] bits.")],
        );
        writeln!(core.stdout, "Supported size: 8, 16, 32, 64, 128.").unwrap();
    }
}

#[cfg(test)]
mod test_print_hex {
    use super::*;
    use rio::*;
    use std::path::Path;
    use test_file::*;
    use writer::Writer;
    use yansi::Paint;

    #[test]
    fn test_help() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let px = PrintHex::new();
        px.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [printHex | px]\n\nUsage:\npx [size]\tView data of at current location in hex format.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    fn test_px_cb(path: &Path) {
        Paint::disable();
        let mut core = Core::new();
        let mut px = PrintHex::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        px.run(&mut core, &["0x0".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 00                                       .\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x2".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001                                     ..\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x3".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 01                                  ...\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x4".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102                                ....\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x5".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 03                             .....\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x6".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305                           ......\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x7".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 08                        .......\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x8".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d                      ........\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x9".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 15                   .........\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xa".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522                 .........\"\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xb".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 37              .........\"7\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xc".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759            .........\"7Y\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xd".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90         .........\"7Y.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xe".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9       .........\"7Y..\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xf".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 79    .........\"7Y..y\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x10".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x11".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db                                       .\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x12".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d                                     .=\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x13".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 18                                  .=.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x14".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855                                .=.U\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x15".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6d                             .=.Um\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x16".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2                           .=.Um.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x17".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2f                        .=.Um./\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x18".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1                      .=.Um./.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x19".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 20                   .=.Um./..\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1a".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011                 .=.Um./...\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1b".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 31              .=.Um./...1\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1c".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142            .=.Um./...1B\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1d".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73         .=.Um./...1Bs\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1e".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5       .=.Um./...1Bs.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1f".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28    .=.Um./...1Bs.(\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x20".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x21".to_string()]);
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

        px.run(&mut core, &["0x22".to_string()]);
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

        px.run(&mut core, &["0x23".to_string()]);
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
        Paint::disable();
        let mut core = Core::new();
        let mut px = PrintHex::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        px.run(&mut core, &["0x0".to_string()]);
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
        px.run(&mut core, &["0x65".to_string()]);
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
        Paint::disable();
        let mut core = Core::new();
        let mut px = PrintHex::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        px.run(&mut core, &[]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "Arguments Error: Expected 1 argument(s), found 0.\n");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: cannot parse integer from empty string\nExpect Hex, binary, Octal or Decimal value but found 0x instead.\n"
        );
    }
}
