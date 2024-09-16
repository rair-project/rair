//! commands handling IO.

mod files;
mod map;
mod print;
mod write;

use self::files::{CloseFile, ListFiles, OpenFile};
use self::map::{ListMap, Map, UnMap};
use self::print::{PrintBase, PrintCSV, PrintHex, PrintSignedCSV};
use self::write::{WriteHex, WriteToFile};
use crate::core::Core;
pub fn register_io(core: &mut Core) {
    let maps = ListMap::new(core);
    let files = ListFiles::new(core);
    let px = PrintHex::new(core);
    core.add_command(Map);
    core.add_command(maps);
    core.add_command(px);
    core.add_command(PrintBase);
    core.add_command(PrintCSV);
    core.add_command(PrintSignedCSV);
    core.add_command(UnMap);
    core.add_command(files);
    core.add_command(OpenFile);
    core.add_command(CloseFile);
    core.add_command(WriteHex);
    core.add_command(WriteToFile);
}
