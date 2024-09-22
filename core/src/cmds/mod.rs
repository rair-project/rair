mod diff;
mod io;
mod loc;
mod traits;
mod utils;

use crate::Core;
pub use traits::*;

pub fn load_commands(core: &mut Core) {
    io::register_io(core);
    loc::register_loc(core);
    utils::register_utils(core);
    diff::register_diff(core);
}
