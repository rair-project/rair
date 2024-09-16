//! Quit the current project.

use crate::{core::Core, Cmd};
use std::process;

#[derive(Default)]
pub struct Quit;

impl Quit {
    pub fn new() -> Self {
        Self
    }
}

impl Cmd for Quit {
    fn run(&mut self, _core: &mut Core, _args: &[String]) {
        process::exit(0);
    }
    fn commands(&self) -> &'static [&'static str] {
        &["quit", "q"]
    }

    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[("", "Quit Current session.")]
    }
}

#[cfg(test)]
mod test_quit {
    use super::*;
    use crate::{writer::Writer, CmdOps};
    #[test]
    fn test_quit_docs() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let quit = Quit::new();
        quit.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [quit | q]\nUsage:\nq\tQuit Current session.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
}
