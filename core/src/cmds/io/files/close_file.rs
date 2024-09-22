use crate::{error_msg, expect, str_to_num, Cmd, Core};

#[derive(Default)]
pub struct CloseFile;

impl CloseFile {
    fn parse_args(core: &mut Core, args: &[String]) -> Option<u64> {
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return None;
        }
        match str_to_num(&args[0]) {
            Ok(hndl) => Some(hndl),
            Err(e) => {
                let err_str = format!("{e}");
                error_msg(core, "Invalid hndl", &err_str);
                None
            }
        }
    }
}

impl Cmd for CloseFile {
    fn commands(&self) -> &'static [&'static str] {
        &["close"]
    }
    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[("[hndl]", "Close file with given hndl.")]
    }
    fn run(&mut self, core: &mut Core, args: &[String]) {
        let Some(hndl) = Self::parse_args(core, args) else {
            return;
        };
        if let Err(e) = core.io.close(hndl) {
            let err_str = format!("{e}");
            error_msg(core, "Failed to close file", &err_str);
        }
    }
}
