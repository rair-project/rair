use crate::{error_msg, expect, str_to_num, Cmd, Core};
use core::fmt::Write;
use std::io::Write as _;

#[derive(Default)]
pub struct PrintBase;

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
        if let Err(e) = core.read(loc, &mut data) {
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
