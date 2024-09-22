use crate::{error_msg, expect, Cmd, Core};

#[derive(Default)]
pub struct WriteHex;

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
                    let msg = format!("{e}.");
                    return error_msg(core, "Failed to parse data", &msg);
                }
            };
            data.push(byte);
        }
        let loc = core.get_loc();
        if let Err(e) = core.write(loc, &data) {
            error_msg(core, "Read Failed", &e.to_string());
        }
    }
    fn commands(&self) -> &'static [&'static str] {
        &["writetHex", "wx"]
    }

    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[(
            "[hexpairs]",
            "write given hexpairs data into the current address.",
        )]
    }
}
