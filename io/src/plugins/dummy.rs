//! RIO dummy plugin: It does nothing at all, usually it is used when reopening files.

use crate::plugin::RIOPluginOperations;
use crate::utils::IoError;
pub struct Dummy;

impl RIOPluginOperations for Dummy {
    fn read(&mut self, _raddr: usize, _buffer: &mut [u8]) -> Result<(), IoError> {
        Ok(())
    }
    fn write(&mut self, _raddr: usize, _buffer: &[u8]) -> Result<(), IoError> {
        Ok(())
    }
}
