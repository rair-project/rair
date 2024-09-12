//! commands handling data writing to files.

use crate::core::*;
use crate::helper::*;
use std::fs::File;
use std::io::prelude::*;

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
            error_msg(
                core,
                "Failed to parse data",
                "Data can't have odd number of digits.",
            );
            return;
        }
        let mut hexpairs = args[0].chars().peekable();
        let mut data = Vec::with_capacity(args.len() / 2);
        while hexpairs.peek().is_some() {
            let chunk: String = hexpairs.by_ref().take(2).collect();
            let byte = match u8::from_str_radix(&chunk, 16) {
                Ok(byte) => byte,
                Err(e) => {
                    let msg = format!("{}.", e);
                    return error_msg(core, "Failed to parse data", &msg);
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
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            "writetHex",
            "wx",
            vec![(
                "[hexpairs]",
                "write given hexpairs data into the current address.",
            )],
        );
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
                let err_str = format!("{}.", e);
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
                let err_str = format!("{}.", e);
                error_msg(core, "Failed to open file", &err_str);
                return;
            }
        };
        if let Err(e) = file.write_all(&data) {
            let err_str = format!("{}.", e);
            error_msg(core, "Failed to write data to file", &err_str);
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            "writeToFile",
            "wtf",
            vec![(
                "[size] [filepath]",
                "write data of size [size] at current location to file identified by [filepath].",
            )],
        );
    }
}

#[cfg(test)]

mod test_write {
    use super::*;
    use crate::writer::Writer;
    use rair_io::*;
    use std::fs;
    #[test]
    fn test_help() {
        let mut core = Core::new_no_colors();
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
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut wx = WriteHex::new();
        core.io
            .open("malloc://0x5000", IoMode::READ | IoMode::WRITE)
            .unwrap();
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
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut wtf = WriteToFile::new();
        core.io
            .open("malloc://0x50", IoMode::READ | IoMode::WRITE)
            .unwrap();
        core.io.map(0x0, 0x500, 0x50).unwrap();

        wtf.run(
            &mut core,
            &["0x50".to_string(), "out_test_wtf_phy".to_string()],
        );
        let mut file = File::open("out_test_wtf_phy").unwrap();
        let mut data = vec![];
        file.read_to_end(&mut data).unwrap();
        assert_eq!(&data[..], &[0u8; 0x50][..]);
        drop(file);
        fs::remove_file("out_test_wtf_phy").unwrap();

        core.set_loc(0x500);
        core.mode = AddrMode::Vir;
        data = vec![];
        wtf.run(
            &mut core,
            &["0x50".to_string(), "out_test_wtf_vir".to_string()],
        );
        file = File::open("out_test_wtf_vir").unwrap();
        file.read_to_end(&mut data).unwrap();
        assert_eq!(&data[..], &[0u8; 0x50][..]);
        drop(file);
        fs::remove_file("out_test_wtf_vir").unwrap();
    }

    #[test]
    fn test_wx_error() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut wx = WriteHex::new();
        core.io
            .open("malloc://0x50", IoMode::READ | IoMode::WRITE)
            .unwrap();
        wx.run(&mut core, &[]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 1 argument(s), found 0.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        wx.run(&mut core, &["012".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse data\nData can't have odd number of digits.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        wx.run(&mut core, &["012x".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse data\ninvalid digit found in string.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.set_loc(0x500);
        wx.run(&mut core, &["0123".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Read Failed\nCannot resolve address.\n"
        );
    }

    #[test]
    fn test_wtf_error() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut wtf = WriteToFile::new();
        core.io
            .open("malloc://0x50", IoMode::READ | IoMode::WRITE)
            .unwrap();
        wtf.run(&mut core, &[]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 2 argument(s), found 0.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        wtf.run(
            &mut core,
            &["0b12".to_string(), "file_that_won't_be_created".to_string()],
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse size\ninvalid digit found in string.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        wtf.run(
            &mut core,
            &[
                "0x1234".to_string(),
                "file_that_won't_be_created".to_string(),
            ],
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to read data\nCannot resolve address.\n"
        );
    }
}
