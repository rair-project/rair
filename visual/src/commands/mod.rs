mod visual_hex;

use rair_core::Core;
use visual_hex::VisualHex;

pub fn register_commands(core: &mut Core) {
    let vx = VisualHex::new(core);
    core.add_command(vx);
}
