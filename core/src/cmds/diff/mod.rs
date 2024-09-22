mod hexdiff;

use crate::Core;

pub fn register_diff(core: &mut Core) {
    let hexdiff = hexdiff::HexDiff::new(core);
    core.add_command(hexdiff);
}
