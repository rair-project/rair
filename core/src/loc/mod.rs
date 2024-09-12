//! commands handling file location.

mod history;
mod mode;
mod seek;
use self::history::History;
use self::mode::*;
use self::seek::*;
use crate::core::Core;
use parking_lot::Mutex;
use std::sync::Arc;

pub fn register_loc(core: &mut Core) {
    let history = Arc::new(Mutex::new(History::new()));
    core.add_command(
        "mode",
        "m",
        Arc::new(Mutex::new(Mode::with_history(history.clone()))),
    );
    core.add_command(
        "seek",
        "s",
        Arc::new(Mutex::new(Seek::with_history(history))),
    );
}
