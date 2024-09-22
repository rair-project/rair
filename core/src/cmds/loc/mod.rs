//! commands handling file location.

mod history;
mod mode;
mod seek;
use self::history::History;
use self::mode::Mode;
use self::seek::Seek;
use crate::core::Core;
use alloc::sync::Arc;
use parking_lot::Mutex;

pub fn register_loc(core: &mut Core) {
    let history = Arc::new(Mutex::new(History::default()));
    core.add_command(Mode::with_history(history.clone()));
    core.add_command(Seek::with_history(history));
}
