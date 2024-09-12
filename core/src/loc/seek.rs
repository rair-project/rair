//! seek forward or backward in file.

use super::history::History;
use crate::core::*;
use crate::helper::*;

#[derive(Default)]
pub struct Seek {
    history: MRc<History>,
}

impl Seek {
    pub(super) fn with_history(history: MRc<History>) -> Self {
        Seek { history }
    }
    fn backward(&mut self, core: &mut Core) {
        if let Some((mode, addr)) = self.history.lock().backward(core) {
            core.mode = mode;
            core.set_loc(addr);
        } else {
            error_msg(core, "Seek Error", "History is empty.");
        }
    }
    fn forward(&mut self, core: &mut Core) {
        if let Some((mode, addr)) = self.history.lock().forward(core) {
            core.mode = mode;
            core.set_loc(addr);
        } else {
            error_msg(core, "Seek Error", "History is empty.");
        }
    }
    fn add_loc(&mut self, core: &mut Core, offset: u64) {
        if let Some(loc) = core.get_loc().checked_add(offset) {
            self.set_loc(core, loc);
        } else {
            error_msg(core, "Seek Error", "Attempt to add with overflow.");
        }
    }
    fn sub_loc(&mut self, core: &mut Core, offset: u64) {
        if let Some(loc) = core.get_loc().checked_sub(offset) {
            self.set_loc(core, loc);
        } else {
            error_msg(core, "Seek Error", "Attempt to subtract with overflow.");
        }
    }
    #[inline]
    fn set_loc(&mut self, core: &mut Core, offset: u64) {
        self.history.lock().add(core);
        core.set_loc(offset);
    }
}

impl Cmd for Seek {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return;
        }
        if args[0] == "-" {
            self.backward(core);
        } else if args[0] == "+" {
            self.forward(core)
        } else if args[0].starts_with('+') {
            match str_to_num(&args[0][1..]) {
                Ok(offset) => self.add_loc(core, offset),
                Err(e) => error_msg(core, "Seek Error", &e.to_string()),
            }
        } else if args[0].starts_with('-') {
            match str_to_num(&args[0][1..]) {
                Ok(offset) => self.sub_loc(core, offset),
                Err(e) => error_msg(core, "Seek Error", &e.to_string()),
            }
        } else {
            match str_to_num(&args[0]) {
                Ok(offset) => self.set_loc(core, offset),
                Err(e) => error_msg(core, "Seek Error", &e.to_string()),
            }
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            "seek",
            "s",
            vec![
                ("+", "\tRedo Seek."),
                ("-", "\tUndo Seek."),
                ("+[offset]", "Increase current loc by offset."),
                ("-[offset]", "Decrease current loc by offset."),
                ("[offset]", "Set current location to offset."),
            ],
        );
    }
}

#[cfg(test)]

mod test_seek {
    use super::*;
    use crate::writer::Writer;
    #[test]
    fn test_docs() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let seek: Seek = Default::default();
        seek.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [seek | s]\n\
             \n\
             Usage:\n\
             s +\t\tRedo Seek.\n\
             s -\t\tUndo Seek.\n\
             s +[offset]\tIncrease current loc by offset.\n\
             s -[offset]\tDecrease current loc by offset.\n\
             s [offset]\tSet current location to offset.\n\
             "
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_seek() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut seek: Seek = Default::default();
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x0);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        seek.run(&mut core, &["+0x5".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x5);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        seek.run(&mut core, &["+0x17".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x1c);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        seek.run(&mut core, &["-12".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x10);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        seek.run(&mut core, &["0b101011".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0b101011);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        seek.run(&mut core, &["-".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x10);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        seek.run(&mut core, &["+".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0b101011);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        seek.run(&mut core, &["+".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0b101011);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Seek Error\nHistory is empty.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        for _ in 0..4 {
            seek.run(&mut core, &["-".to_string()]);
        }
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0b0);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        seek.run(&mut core, &["-".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Seek Error\nHistory is empty.\n"
        );
    }
    #[test]
    fn test_seek_overflow() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut seek: Seek = Default::default();
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x0);
        seek.run(&mut core, &["-0x5".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x0);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Seek Error\nAttempt to subtract with overflow.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        seek.run(&mut core, &["0xffffffffffffffff".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0xffffffffffffffff);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        seek.run(&mut core, &["+1".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0xffffffffffffffff);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Seek Error\nAttempt to add with overflow.\n"
        );
    }

    #[test]
    fn test_seek_invalid_arguments() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut seek: Seek = Default::default();
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x0);

        seek.run(&mut core, &[]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x0);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 1 argument(s), found 0.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        seek.run(&mut core, &["+ff".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x0);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Seek Error\ninvalid digit found in string\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        seek.run(&mut core, &["-ff".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x0);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Seek Error\ninvalid digit found in string\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        seek.run(&mut core, &["ff".to_string()]);
        assert_eq!(core.mode, AddrMode::Phy);
        assert_eq!(core.get_loc(), 0x0);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Seek Error\ninvalid digit found in string\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
    }
}
