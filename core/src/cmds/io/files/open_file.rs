use crate::{error_msg, expect_range, str_to_num, Cmd, Core};
use rair_io::IoMode;

#[derive(Default)]
pub struct OpenFile;

impl OpenFile {
    fn parse_three_args<'a>(
        core: &mut Core,
        arg0: &str,
        arg1: &'a str,
        arg2: &str,
    ) -> Option<(&'a str, IoMode, Option<u64>)> {
        let perm: IoMode = match arg0.try_into() {
            Ok(perm) => perm,
            Err(e) => {
                error_msg(core, "Failed to parse permission", &e);
                return None;
            }
        };
        let addr = match str_to_num(arg2) {
            Ok(addr) => Some(addr),
            Err(e) => {
                let err_str = format!("{e}");
                error_msg(core, "Failed to parse address", &err_str);
                return None;
            }
        };
        Some((arg1, perm, addr))
    }
    fn parse_two_args<'a>(
        core: &mut Core,
        arg0: &'a str,
        arg1: &'a str,
    ) -> Option<(&'a str, IoMode, Option<u64>)> {
        if let Ok(a) = str_to_num(arg1) {
            Some((arg0, IoMode::READ, Some(a)))
        } else {
            let perm: IoMode = match arg0.try_into() {
                Ok(perm) => perm,
                Err(e) => {
                    error_msg(core, "Failed to parse permission", &e);
                    return None;
                }
            };
            Some((arg1, perm, None))
        }
    }
    fn parse_args<'a>(
        core: &mut Core,
        args: &'a [String],
    ) -> Option<(&'a str, IoMode, Option<u64>)> {
        if args.len() > 3 || args.is_empty() {
            expect_range(core, args.len() as u64, 1, 2);
            return None;
        }
        if args.len() == 3 {
            Self::parse_three_args(core, &args[0], &args[1], &args[2])
        } else if args.len() == 2 {
            Self::parse_two_args(core, &args[0], &args[1])
        } else {
            Some((&args[0], IoMode::READ, None))
        }
    }
}

impl Cmd for OpenFile {
    fn commands(&self) -> &'static [&'static str] {
        &["o", "open"]
    }
    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[("<Perm> [URI] <Addr>", "Open given URI using given optional permission (default to readonly) at given optional address.")]
    }
    fn run(&mut self, core: &mut Core, args: &[String]) {
        let Some((uri, perm, addr)) = Self::parse_args(core, args) else {
            return;
        };
        let result = match addr {
            Some(addr) => core.io.open_at(uri, perm, addr),
            None => core.io.open(uri, perm),
        };
        if let Err(e) = result {
            let err_str = format!("{e}");
            error_msg(core, "Failed to open file", &err_str);
        }
    }
}
