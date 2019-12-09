/*
 * write: commands handling data writing to files.
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
use std::fs::File;
use std::io::prelude::*;
use std::u8;

#[derive(Default)]
pub struct WriteHex {}

impl WriteHex {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for WriteHex {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return;
        }
        if args[0].len() % 2 != 0 {
            error_msg(core, "Failed to hexpairs", "Data can't have odd number of digits.");
            return;
        }
        let mut hexpairs = args[0].chars().peekable();
        let mut data = Vec::with_capacity(args.len() / 2);
        while hexpairs.peek().is_some() {
            let chunk: String = hexpairs.by_ref().take(2).collect();
            let byte = match u8::from_str_radix(&chunk, 16) {
                Ok(byte) => byte,
                Err(e) => {
                    let msg = format!("{}", e);
                    return error_msg(core, "Failed to hexpairs", &msg);
                }
            };
            data.push(byte);
        }
        let loc = core.get_loc();
        let error = match core.mode {
            AddrMode::Phy => core.io.pwrite(loc, &data),
            AddrMode::Vir => core.io.vwrite(loc, &data),
        };
        if let Err(e) = error {
            error_msg(core, "Read Failed", &e.to_string());
            return;
        }
    }
    fn help(&self, core: &mut Core) {
        help(core, &"writetHex", &"wx", vec![("[hexpairs]", "write given hexpairs data into the current address.")]);
    }
}

#[derive(Default)]
pub struct WriteToFile {}

impl WriteToFile {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for WriteToFile {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 2 {
            expect(core, args.len() as u64, 2);
            return;
        }
        let size = match str_to_num(&args[0]) {
            Ok(size) => size as usize,
            Err(e) => {
                let err_str = format!("{}", e);
                error_msg(core, "Failed to parse size", &err_str);
                return;
            }
        };
        let loc = core.get_loc();
        let mut data = vec![0; size];
        let error = match core.mode {
            AddrMode::Phy => core.io.pread(loc, &mut data),
            AddrMode::Vir => core.io.vread(loc, &mut data),
        };
        if let Err(e) = error {
            error_msg(core, "Failed to read data", &e.to_string());
            return;
        }
        let mut file = match File::create(&args[1]) {
            Ok(file) => file,
            Err(e) => {
                let err_str = format!("{}", e);
                error_msg(core, "Failed to open file", &err_str);
                return;
            }
        };
        if let Err(e) = file.write_all(&data) {
            let err_str = format!("{}", e);
            error_msg(core, "Failed to write data to file", &err_str);
            return;
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            &"writeToFile",
            &"wtf",
            vec![("[size] [filepath]", "write data of size [size] at current location to file identified by [filepath].")],
        );
    }
}

#[cfg(test)]

mod test_write {
    use super::*;
    use rio::*;
    use std::fs;
    use writer::Writer;
    use yansi::Paint;
    #[test]
    fn test_help() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let wx = WriteHex::new();
        let wtf = WriteToFile::new();
        wx.help(&mut core);
        wtf.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [writetHex | wx]\n\n\
             Usage:\n\
             wx [hexpairs]\twrite given hexpairs data into the current address.\n\
             Commands: [writeToFile | wtf]\n\n\
             Usage:\n\
             wtf [size] [filepath]\twrite data of size [size] at current location to file identified by [filepath].\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_wx() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut wx = WriteHex::new();
        core.io.open("malloc://0x5000", IoMode::READ | IoMode::WRITE).unwrap();
        core.io.map(0x0, 0x500, 0x500).unwrap();
        wx.run(&mut core, &["123456789abcde".to_string()]);
        let mut data = [0; 7];
        core.io.pread(0x0, &mut data).unwrap();
        assert_eq!(data, [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde]);
        core.set_loc(0x500);
        core.mode = AddrMode::Vir;
        wx.run(&mut core, &["23456789abcde1".to_string()]);
        core.io.vread(0x500, &mut data).unwrap();
        assert_eq!(data, [0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xe1]);
    }

    #[test]
    fn test_wtf() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut wtf = WriteToFile::new();
        core.io.open("malloc://0x50", IoMode::READ | IoMode::WRITE).unwrap();
        core.io.map(0x0, 0x500, 0x50).unwrap();

        wtf.run(&mut core, &["0x50".to_string(), "out_test_wtf_phy".to_string()]);
        let mut file = File::open("out_test_wtf_phy").unwrap();
        let mut data = vec![];
        file.read_to_end(&mut data).unwrap();
        assert_eq!(&data[..], &[0u8; 0x50][..]);
        drop(file);
        fs::remove_file("out_test_wtf_phy").unwrap();

        core.set_loc(0x500);
        core.mode = AddrMode::Vir;
        data = vec![];
        wtf.run(&mut core, &["0x50".to_string(), "out_test_wtf_vir".to_string()]);
        file = File::open("out_test_wtf_vir").unwrap();
        file.read_to_end(&mut data).unwrap();
        assert_eq!(&data[..], &[0u8; 0x50][..]);
        drop(file);
        fs::remove_file("out_test_wtf_vir").unwrap();
    }
}
