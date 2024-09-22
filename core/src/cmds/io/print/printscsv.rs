use crate::{error_msg, expect, str_to_num, Cmd, Core};
use core::fmt::Write;
use std::io::Write as _;

#[derive(Default)]
pub struct PrintSignedCSV;

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
        if let Err(e) = core.read(loc, &mut data) {
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
