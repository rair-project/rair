//! Quit the current project.

use crate::core::*;
use crate::helper::*;
use std::process;

#[derive(Default)]
pub struct Quit {}

impl Quit {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for Quit {
    fn run(&mut self, _core: &mut Core, _args: &[String]) {
        process::exit(0);
    }
    fn help(&self, core: &mut Core) {
        help(core, "quit", "q", vec![("", "Quit Current session.")]);
    }
}

#[cfg(test)]
mod test_quit {
    use super::*;
    use crate::writer::Writer;
    #[test]
    fn test_quit_docs() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let quit = Quit::new();
        quit.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [quit | q]\n\nUsage:\nq\tQuit Current session.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
}
