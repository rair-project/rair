//! RIO interface for implementing new plugin.

use crate::utils::{IoError, IoMode};

/// Metadata that describes the plugin
#[derive(PartialEq)]
pub struct RIOPluginMetadata {
    /// Name of the plugin
    pub name: &'static str,
    /// Short description of the plugin
    pub desc: &'static str,
    /// Name of the author of the plugin
    pub author: &'static str,
    /// License of the plugin
    pub license: &'static str,
    /// Version of the plugin
    pub version: &'static str,
}

/// This class is populated via [`RIOPlugin::open`]
pub struct RIOPluginDesc {
    /// URI to be opened
    pub name: String,
    /// Permissions which is opened with
    pub perm: IoMode,
    /// real base physical address of the file
    pub raddr: u64, //padd is simulated physical address
    /// Size of the file
    pub size: u64,
    /// object that implements read/write on the file
    pub plugin_operations: Box<dyn RIOPluginOperations + Sync + Send>,
}

/// This trait should be implemented by object that allows plugin to open files or check metadata
/// of the plugin.
pub trait RIOPlugin {
    /// Retrieve reference to the plugin metadata
    fn get_metadata(&self) -> &'static RIOPluginMetadata;
    /// Open a file given a uri (<extension://file> path) using the mode specified by flags.
    fn open(&mut self, uri: &str, flags: IoMode) -> Result<RIOPluginDesc, IoError>;
    /// Check if the given file can be opened wit the current plugin (only by checking the uri
    /// without opening the file)
    fn accept_uri(&self, uri: &str) -> bool;
}
/// A call to [`RIOPlugin::open`] would normally return [`RioPluginDesc`] that contains member that
/// implements [`RIOPluginOperations`]. This way we always have way of reading and writing from file
/// with custom data encoding.
pub trait RIOPluginOperations {
    /// Function that read from a file represented by an object opened
    /// by [`RIOPlugin::open`] raddr is the real address of the in the file.
    fn read(&mut self, raddr: usize, buffer: &mut [u8]) -> Result<(), IoError>;
    /// Function that writes to a file represented by an object opened
    /// by [`RIOPlugin::open`] raddr is the real address of the in the file.
    fn write(&mut self, raddr: usize, buffer: &[u8]) -> Result<(), IoError>;
}

struct DefPluginOperations;
impl RIOPluginOperations for DefPluginOperations {
    fn read(&mut self, _raddr: usize, _buffer: &mut [u8]) -> Result<(), IoError> {
        Ok(())
    }
    fn write(&mut self, _raddr: usize, _buffer: &[u8]) -> Result<(), IoError> {
        Ok(())
    }
}

impl Default for Box<dyn RIOPluginOperations + Sync + Send> {
    fn default() -> Self {
        Box::new(DefPluginOperations)
    }
}
