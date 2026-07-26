use crate::Core;
use std::io::Write as _;
use yansi::Paint as _;

pub trait Cmd {
    fn commands(&self) -> &'static [&'static str];
    fn help_messages(&self) -> &'static [(&'static str, &'static str)];
    fn run(&mut self, _: &mut Core, _: &[String]);
}

pub trait CmdOps: Cmd {
    fn help(&self, core: &mut Core);
    fn sorted_commands(&self) -> Vec<&'static str>;
}

impl<T: ?Sized + Cmd> CmdOps for T {
    fn help(&self, core: &mut Core) {
        let (r1, g1, b1) = core.env.read().get_color("color.6").unwrap();
        let (r2, g2, b2) = core.env.read().get_color("color.7").unwrap();
        let commands = self.sorted_commands();
        if commands.len() == 1 {
            write!(core.stdout, "Command: [").unwrap();
        } else {
            write!(core.stdout, "Commands: [").unwrap();
        }
        let mut iter = commands.iter().rev().map(|cmd| cmd.rgb(r1, g1, b1));
        write!(core.stdout, "{}", iter.next().unwrap()).unwrap();
        for c in iter {
            write!(core.stdout, " | {c}").unwrap();
        }
        writeln!(core.stdout, "]\nUsage:").unwrap();
        for (args, description) in self.help_messages() {
            write!(core.stdout, "{}", commands[0].rgb(r1, g1, b1)).unwrap();
            if !args.is_empty() {
                write!(core.stdout, " {}", args.rgb(r2, g2, b2)).unwrap();
            }
            writeln!(core.stdout, "\t{description}").unwrap();
        }
    }

    fn sorted_commands(&self) -> Vec<&'static str> {
        let mut commands = self.commands().to_vec();
        commands.sort_by_key(|a| a.len());
        commands
    }
}

#[cfg(test)]
mod cmd_tests {
    use super::Cmd;
    use crate::{CmdOps as _, Core, Writer};

    struct LongShort;
    impl Cmd for LongShort {
        fn commands(&self) -> &'static [&'static str] {
            &["Test", "t"]
        }
        fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
            &[("t1", "test 1"), ("t2", "test 2")]
        }
        fn run(&mut self, _: &mut crate::Core, _: &[String]) {
            // never invoked in these tests
        }
    }

    struct Long;

    impl Cmd for Long {
        fn commands(&self) -> &'static [&'static str] {
            &["Test"]
        }
        fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
            &[("t1", "test 1"), ("t2", "test 2")]
        }
        fn run(&mut self, _: &mut Core, _: &[String]) {
            // never invoked in these tests
        }
    }
    #[test]
    fn help_short() {
        let mut core = Core::new_no_colors();
        core.stdout = Writer::new_buf();
        yansi::disable();
        let ls = LongShort;
        ls.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [Test | t]\nUsage:\nt t1\ttest 1\nt t2\ttest 2\n"
        );
    }
    #[test]
    fn help_long() {
        let mut core = Core::new_no_colors();
        core.stdout = Writer::new_buf();
        yansi::disable();
        let l = Long;
        l.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Command: [Test]\nUsage:\nTest t1\ttest 1\nTest t2\ttest 2\n"
        );
    }
}
