//! Helper functions for implementing external or internal commands.

use crate::core::Core;
use alloc::fmt;
use alloc::sync::Arc;
use core::fmt::Display;
use parking_lot::Mutex;
use rair_env::Environment;
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use std::num;
use std::process::exit;
use yansi::Paint as _;

pub type MRc<T> = Arc<Mutex<T>>; //mutable refcounter that is thread safe

#[expect(
    clippy::exhaustive_structs,
    reason = "`CmdFunctions` is deliberately a plain pair of function pointers constructible by API users"
)]
pub struct CmdFunctions {
    pub help: fn(&mut Core),
    pub run: fn(&mut Core, &[String]),
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AddrMode {
    Phy,
    Vir,
}

impl Display for AddrMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddrMode::Phy => write!(f, "Phy"),
            AddrMode::Vir => write!(f, "Vir"),
        }
    }
}

/// Converts `n` to a `u64`. The numeral base is detected from the prefix:
/// `0b` for binary, `0x` for hexadecimal, a leading `0` for octal, and
/// decimal otherwise.
///
/// # Errors
///
/// Returns [`num::ParseIntError`] if `n` is not a valid numeral in the
/// detected base or if the parsed value does not fit into a `u64`.
pub fn str_to_num(n: &str) -> Result<u64, num::ParseIntError> {
    if let Some(prefix) = n.get(0..2) {
        match &*prefix.to_lowercase() {
            "0b" => return u64::from_str_radix(n.get(2..).unwrap_or_default(), 2),
            "0x" => return u64::from_str_radix(n.get(2..).unwrap_or_default(), 16),
            _ => (),
        }
    }
    if n.len() > 1 && n.starts_with('0') {
        return u64::from_str_radix(n.get(1..).unwrap_or_default(), 8);
    }
    n.parse::<u64>()
}

/// Prints an error message to `core.stderr` telling the user that exactly
/// `expect` arguments were expected but `args_len` were found.
///
/// # Panics
///
/// Panics if the `color.4` environment variable is missing or is not a
/// color, or if writing to `core.stderr` fails.
pub fn expect(core: &mut Core, args_len: u64, expect: u64) {
    let (r, g, b) = core.env.read().get_color("color.4").unwrap();
    let error = "Arguments Error";
    let expected = format!("{expect}");
    let found = format!("{args_len}");
    writeln!(
        core.stderr,
        "{}: Expected {} argument(s), found {}.",
        error.rgb(r, g, b).bold(),
        expected.rgb(r, g, b),
        found.rgb(r, g, b)
    )
    .unwrap();
}

/// Prints an error message to `core.stderr` telling the user that between
/// `min` and `max` arguments were expected but `args_len` were found.
///
/// # Panics
///
/// Panics if `min >= max`, if the `color.4` environment variable is missing
/// or is not a color, or if writing to `core.stderr` fails.
pub fn expect_range(core: &mut Core, args_len: u64, min: u64, max: u64) {
    assert!(min < max, "`expect_range` requires `min` < `max`");
    let (r, g, b) = core.env.read().get_color("color.4").unwrap();
    let error = "Arguments Error";
    let min_str = format!("{min}");
    let max_str = format!("{max}");
    let found = format!("{args_len}");
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

/// Prints a `title`d error message followed by the detailed `msg` to
/// `core.stderr`.
///
/// # Panics
///
/// Panics if the `color.4` environment variable is missing or is not a
/// color, or if writing to `core.stderr` fails.
pub fn error_msg(core: &mut Core, title: &str, msg: &str) {
    let (r, g, b) = core.env.read().get_color("color.4").unwrap();
    writeln!(
        core.stderr,
        "{}: {}",
        "Error".rgb(r, g, b).bold(),
        title.rgb(r, g, b)
    )
    .unwrap();
    writeln!(core.stderr, "{msg}").unwrap();
}

/// Reports an unrecoverable error described by `title` and `msg` on
/// `core.stderr` and terminates the process. This function never returns:
/// it is the documented abort path for unrecoverable errors.
///
/// # Panics
///
/// Panics if the `color.4` environment variable is missing or is not a
/// color, or if writing to `core.stderr` fails; otherwise the process is
/// terminated via `exit`.
#[expect(
    clippy::exit,
    reason = "panic_msg is the documented abort path for unrecoverable errors"
)]
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
        writeln!(core.stderr, "{msg}").unwrap();
    }
    writeln!(core.stderr, "{}", "Exiting!".rgb(r, g, b).bold()).unwrap();
    exit(-1);
}

#[must_use]
pub fn is_color<Core>(_: &str, value: &str, env: &Environment<Core>, _: &mut Core) -> bool {
    env.is_color(value)
}

#[cfg(test)]
mod test_helper {
    use super::*;
    use crate::writer::Writer;
    use core::fmt::Write as _;
    #[test]
    fn str_to_num_works() {
        assert_eq!(str_to_num("12345").unwrap(), 12345);
        assert_eq!(str_to_num("012345").unwrap(), 0o12345);
        assert_eq!(str_to_num("0b101001").unwrap(), 0b10_1001);
        assert_eq!(str_to_num("0x12345").unwrap(), 0x12345);
        assert_eq!(str_to_num("0X1F2f345").unwrap(), 0x01f2_f345);
        assert_eq!(str_to_num("0").unwrap(), 0);
        str_to_num("0x12345123451234512").unwrap_err();
    }

    #[test]
    fn except() {
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
    fn expect_range_works() {
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
    fn error_msg_works() {
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
    fn addr_mode() {
        let mut s = String::new();
        write!(s, "{} {}", AddrMode::Phy, AddrMode::Vir).unwrap();
        assert_eq!(s, "Phy Vir");
    }
}
