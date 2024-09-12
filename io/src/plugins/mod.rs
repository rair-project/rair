//! List of built-in RIO plugins.

use crate::io::RIO;
pub mod base64;
pub mod defaultplugin;
pub mod dummy;
pub mod ihex;
pub mod malloc;
pub mod srec;
pub(crate) fn load_plugins(io: &mut RIO) {
    io.load_plugin(defaultplugin::plugin());
    io.load_plugin(ihex::plugin());
    io.load_plugin(malloc::plugin());
    io.load_plugin(base64::plugin());
    io.load_plugin(srec::plugin());
}
