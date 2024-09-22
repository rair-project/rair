use crate::{error_msg, expect, str_to_num, Cmd, Core};
use std::{fs::File, io::Write};

#[derive(Default)]
pub struct WriteToFile;

impl Cmd for WriteToFile {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 2 {
            expect(core, args.len() as u64, 2);
            return;
        }
        let size = match str_to_num(&args[0]) {
            Ok(size) => size as usize,
            Err(e) => {
                let err_str = format!("{e}.");
                error_msg(core, "Failed to parse size", &err_str);
                return;
            }
        };
        let loc = core.get_loc();
        let mut data = vec![0; size];
        if let Err(e) = core.read(loc, &mut data) {
            error_msg(core, "Failed to read data", &e.to_string());
            return;
        }
        let mut file = match File::create(&args[1]) {
            Ok(file) => file,
            Err(e) => {
                let err_str = format!("{e}.");
                error_msg(core, "Failed to open file", &err_str);
                return;
            }
        };
        if let Err(e) = file.write_all(&data) {
            let err_str = format!("{e}.");
            error_msg(core, "Failed to write data to file", &err_str);
        }
    }

    fn commands(&self) -> &'static [&'static str] {
        &["writeToFile", "wtf"]
    }

    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[(
            "[size] [filepath]",
            "write data of size [size] at current location to file identified by [filepath].",
        )]
    }
}
