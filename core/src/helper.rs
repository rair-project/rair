//! Helper functions for implementing external or internal commands.

use crate::core::*;
use parking_lot::Mutex;
use rair_env::Environment;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::io::Write;
use std::num;
use std::process::exit;
use std::sync::Arc;
use yansi::Paint;

pub type MRc<T> = Arc<Mutex<T>>; //mutable refcounter that is thread safe

pub fn str_to_num(n: &str) -> Result<u64, num::ParseIntError> {
    if n.len() >= 2 {
        match &*n[0..2].to_lowercase() {
            "0b" => return u64::from_str_radix(&n[2..], 2),
            "0x" => return u64::from_str_radix(&n[2..], 16),
            _ => (),
        }
    }
    if n.len() > 1 && n.starts_with('0') {
        return u64::from_str_radix(&n[1..], 8);
    }
    n.parse::<u64>()
}

pub fn expect(core: &mut Core, args_len: u64, expect: u64) {
    let (r, g, b) = core.env.read().get_color("color.4").unwrap();
    let error = "Arguments Error";
    let expected = format!("{}", expect);
    let found = format!("{}", args_len);
    writeln!(
        core.stderr,
        "{}: Expected {} argument(s), found {}.",
        error.rgb(r, g, b).bold(),
        expected.rgb(r, g, b),
        found.rgb(r, g, b)
    )
    .unwrap();
}

pub fn expect_range(core: &mut Core, args_len: u64, min: u64, max: u64) {
    assert!(min < max);
    let (r, g, b) = core.env.read().get_color("color.4").unwrap();
    let error = "Arguments Error";
    let min_str = format!("{}", min);
    let max_str = format!("{}", max);
    let found = format!("{}", args_len);
    writeln!(
        core.stderr,
        "{}: Expected between {} and {} arguments, found {}.",
        error.rgb(r, g, b).bold(),
        min_str.rgb(r, g, b),
        max_str.rgb(r, g, b),
        found.rgb(r, g, b)
    )
    .unwrap();
}

pub fn error_msg(core: &mut Core, title: &str, msg: &str) {
    let (r, g, b) = core.env.read().get_color("color.4").unwrap();
    writeln!(
        core.stderr,
        "{}: {}",
        "Error".rgb(r, g, b).bold(),
        title.rgb(r, g, b)
    )
    .unwrap();
    writeln!(core.stderr, "{}", msg).unwrap();
}

pub fn panic_msg(core: &mut Core, title: &str, msg: &str) -> ! {
    let (r, g, b) = core.env.read().get_color("color.4").unwrap();
    writeln!(
        core.stderr,
        "{}: {}",
        "Unrecoverable Error".rgb(r, g, b).bold(),
        title.rgb(r, g, b)
    )
    .unwrap();
    if !msg.is_empty() {
        writeln!(core.stderr, "{}", msg).unwrap();
    }
    writeln!(core.stderr, "{}", "Exiting!".rgb(r, g, b).bold()).unwrap();
    exit(-1);
}

pub fn help(core: &mut Core, long: &str, short: &str, usage: Vec<(&str, &str)>) {
    let (r1, g1, b1) = core.env.read().get_color("color.6").unwrap();
    let (r2, g2, b2) = core.env.read().get_color("color.7").unwrap();
    let used = if short.is_empty() {
        writeln!(core.stdout, "Command: [{}]\n", long.rgb(r1, g1, b1)).unwrap();
        long
    } else {
        writeln!(
            core.stdout,
            "Commands: [{} | {}]\n",
            long.rgb(r1, g1, b1),
            short.rgb(r1, g1, b1)
        )
        .unwrap();
        short
    };
    writeln!(core.stdout, "Usage:").unwrap();
    for (args, description) in usage {
        write!(core.stdout, "{}", used.rgb(r1, g1, b1)).unwrap();
        if !args.is_empty() {
            write!(core.stdout, " {}", args.rgb(r2, g2, b2)).unwrap();
        }
        writeln!(core.stdout, "\t{}", description,).unwrap()
    }
}

pub struct CmdFunctions {
    pub run: fn(&mut Core, &[String]),
    pub help: fn(&mut Core),
}

pub trait Cmd {
    fn run(&mut self, _: &mut Core, _: &[String]);
    fn help(&self, _: &mut Core);
}
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum AddrMode {
    Vir,
    Phy,
}

impl Display for AddrMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddrMode::Phy => write!(f, "Phy"),
            AddrMode::Vir => write!(f, "Vir"),
        }
    }
}

pub fn is_color<Core>(_: &str, value: &str, env: &Environment<Core>, _: &mut Core) -> bool {
    env.is_color(value)
}

#[cfg(test)]
mod test_helper {
    use super::*;
    use crate::writer::Writer;
    use std::fmt::Write;
    #[test]
    fn test_str_to_num() {
        assert_eq!(str_to_num("12345").unwrap(), 12345);
        assert_eq!(str_to_num("012345").unwrap(), 0o12345);
        assert_eq!(str_to_num("0b101001").unwrap(), 0b101001);
        assert_eq!(str_to_num("0x12345").unwrap(), 0x12345);
        assert_eq!(str_to_num("0X1F2f345").unwrap(), 0x1f2f345);
        assert_eq!(str_to_num("0").unwrap(), 0);
        assert!(str_to_num("0x12345123451234512").is_err());
    }

    #[test]
    fn test_except() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        yansi::disable();
        expect(&mut core, 5, 7);
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 7 argument(s), found 5.\n"
        );
    }
    #[test]
    fn test_expect_range() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        yansi::disable();
        expect_range(&mut core, 5, 7, 10);
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected between 7 and 10 arguments, found 5.\n"
        );
    }

    #[test]
    fn test_error_msg() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        yansi::disable();
        error_msg(&mut core, "Error Title", "Something might have failed.");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Error Title\nSomething might have failed.\n"
        );
    }
    #[test]
    fn test_help_short() {
        let mut core = Core::new_no_colors();
        core.stdout = Writer::new_buf();
        yansi::disable();
        help(
            &mut core,
            "Test",
            "t",
            vec![("t1", "test 1"), ("t2", "test 2")],
        );
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [Test | t]\n\nUsage:\nt t1\ttest 1\nt t2\ttest 2\n"
        );
    }
    #[test]
    fn test_help_long() {
        let mut core = Core::new_no_colors();
        core.stdout = Writer::new_buf();
        yansi::disable();
        help(
            &mut core,
            "Test",
            "",
            vec![("t1", "test 1"), ("t2", "test 2")],
        );
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Command: [Test]\n\nUsage:\nTest t1\ttest 1\nTest t2\ttest 2\n"
        );
    }
    #[test]
    fn test_addr_mode() {
        let mut s = String::new();
        write!(s, "{} {}", AddrMode::Phy, AddrMode::Vir).unwrap();
        assert_eq!(s, "Phy Vir");
    }
}
