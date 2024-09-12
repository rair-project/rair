/*
 * desc.rs: file descriptor data structure and needed tools to operate on single file.
 * Copyright (C) 2019  Oddcoder
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use crate::plugin::*;
use crate::utils::*;
use serde::{Deserialize, Serialize};

/// This struct represents a file that is opened in [RIO]
#[derive(Serialize, Deserialize)]
pub struct RIODesc {
    pub(crate) name: String,
    pub(crate) perm: IoMode,
    pub(crate) hndl: u64,
    pub(crate) paddr: u64, //padd is simulated physical address
    pub(crate) size: u64,
    raddr: u64, // raddr is the IO descriptor address, general rule of interaction paddr is high level lie, while raddr is the real thing.
    // Since we are skiping files operation structures .. after deserializing RIO .. we must
    // reopen the files again and make sure that they are in the right place
    // for sake of serde skip Box<dyn RIOPluginOperations + Sync + Send> must implement Default and
    // the implementation is found in plugins.rs
    #[serde(skip)]
    plugin_operations: Box<dyn RIOPluginOperations + Sync + Send>,
}

impl RIODesc {
    pub(crate) fn raddr(&self) -> u64 {
        self.raddr
    }
    pub(crate) fn open(plugin: &mut dyn RIOPlugin, uri: &str, flags: IoMode) -> Result<RIODesc, IoError> {
        let plugin_desc = plugin.open(uri, flags)?;
        let desc = RIODesc {
            hndl: 0,
            name: plugin_desc.name,
            perm: plugin_desc.perm,
            paddr: 0,
            size: plugin_desc.size,
            plugin_operations: plugin_desc.plugin_operations,
            raddr: plugin_desc.raddr,
        };
        Ok(desc)
    }
    pub(crate) fn reopen(&mut self, plugin: &mut dyn RIOPlugin) -> Result<(), IoError> {
        let plugin_desc = plugin.open(&self.name, self.perm)?;
        self.plugin_operations = plugin_desc.plugin_operations;
        self.raddr = plugin_desc.raddr;
        Ok(())
    }
    pub(crate) fn read(&mut self, paddr: usize, buffer: &mut [u8]) -> Result<(), IoError> {
        self.plugin_operations.read(paddr - self.paddr as usize + self.raddr as usize, buffer)
    }
    pub(crate) fn write(&mut self, paddr: usize, buffer: &[u8]) -> Result<(), IoError> {
        self.plugin_operations.write(paddr - self.paddr as usize + self.raddr as usize, buffer)
    }
    /// Returns URI of current file descriptor.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns *true* if paddr exists in this file descriptor and *false* otherwise.
    pub fn has_paddr(&self, paddr: u64) -> bool {
        paddr >= self.paddr && paddr < self.paddr + self.size
    }
    /// Returns the base physical address of this file.
    pub fn paddr_base(&self) -> u64 {
        self.paddr
    }
    /// Returns size of file on disk.
    pub fn size(&self) -> u64 {
        self.size
    }
    /// Returns the permissions which the file was opened with.
    pub fn perm(&self) -> IoMode {
        self.perm
    }
    /// Returns the Handle of given file descriptor.
    pub fn hndl(&self) -> u64 {
        self.hndl
    }
}

#[cfg(test)]
mod default_plugin_tests {
    use super::*;
    use crate::plugins::defaultplugin;
    use std::io;
    use std::path::Path;
    use test_file::*;
    fn test_desc_read_cb(path: &Path) {
        let mut plugin = defaultplugin::plugin();
        let mut desc = RIODesc::open(&mut *plugin, &path.to_string_lossy(), IoMode::READ).unwrap();
        desc.paddr = 0x40000;
        let mut buffer: &mut [u8] = &mut [0; 8];
        // read at the begining
        desc.read(desc.paddr as usize, buffer).unwrap();
        assert_eq!(buffer, [0x00, 0x01, 0x01, 0x02, 0x03, 0x05, 0x08, 0x0d]);
        // read at the middle
        desc.read((desc.paddr + 0x10) as usize, buffer).unwrap();
        assert_eq!(buffer, [0xdb, 0x3d, 0x18, 0x55, 0x6d, 0xc2, 0x2f, 0xf1]);
        // read at the end
        desc.read((desc.paddr + 97) as usize, buffer).unwrap();
        assert_eq!(buffer, [0x41, 0xc1, 0x02, 0xc3, 0xc5, 0x88, 0x4d, 0xd5]);
    }
    #[test]
    fn test_desc_read() {
        operate_on_file(&test_desc_read_cb, DATA)
    }
    fn test_desc_has_paddr_cb(path: &Path) {
        let mut plugin = defaultplugin::plugin();
        let mut desc = RIODesc::open(&mut *plugin, &path.to_string_lossy(), IoMode::READ).unwrap();
        desc.paddr = 0x40000;
        assert!(desc.has_paddr(0x40000));
        assert!(!desc.has_paddr(0x5));
        assert!(!desc.has_paddr(0x40000 + DATA.len() as u64));
        assert!(desc.has_paddr(0x40000 + DATA.len() as u64 - 1));
    }
    #[test]
    fn test_desc_has_paddr() {
        operate_on_file(&test_desc_has_paddr_cb, DATA);
    }
    fn test_desc_read_errors_cb(path: &Path) {
        let mut plugin = defaultplugin::plugin();
        let mut desc = RIODesc::open(&mut *plugin, &path.to_string_lossy(), IoMode::READ).unwrap();
        desc.paddr = 0x40000;
        let mut buffer: &mut [u8] = &mut [0; 8];
        // read past the end
        let mut e = desc.read((desc.paddr + desc.size) as usize, buffer);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(true, "UnexpectedEof Error should have been generated"),
        };
        // read at the middle past the the end
        e = desc.read((desc.paddr + desc.size - 5) as usize, buffer);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(true, "UnexpectedEof Error should have been generated"),
        };

        // read at the start past the end
        let mut v: Vec<u8> = vec![0; (desc.size + 8) as usize];
        buffer = &mut v;
        e = desc.read(desc.paddr as usize, buffer);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(true, "UnexpectedEof Error should have been generated"),
        };
    }
    #[test]
    fn test_desc_read_errors() {
        operate_on_file(&test_desc_read_errors_cb, DATA);
    }

    fn test_desc_write_cb(path: &Path) {
        let mut plugin = defaultplugin::plugin();
        let mut desc = RIODesc::open(&mut *plugin, &path.to_string_lossy(), IoMode::READ | IoMode::WRITE).unwrap();
        let mut buffer: &mut [u8] = &mut [0; 8];
        desc.paddr = 0x40000;
        // write at the begining
        desc.write(desc.paddr as usize, buffer).unwrap();
        desc.read(desc.paddr as usize, buffer).unwrap();
        assert_eq!(buffer, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // write at the middle
        desc.write((desc.paddr + 0x10) as usize, buffer).unwrap();
        desc.read((desc.paddr + 0x10) as usize, buffer).unwrap();
        assert_eq!(buffer, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // write at the end
        desc.write((desc.paddr + 97) as usize, buffer).unwrap();
        desc.read((desc.paddr + 97) as usize, buffer).unwrap();
        assert_eq!(buffer, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_desc_write() {
        operate_on_file(&test_desc_write_cb, DATA);
    }

    fn test_write_errors_cb(path: &Path) {
        let mut plugin = defaultplugin::plugin();
        let mut desc = RIODesc::open(&mut *plugin, &path.to_string_lossy(), IoMode::READ | IoMode::WRITE).unwrap();
        let mut buffer: &[u8] = &[0; 8];
        desc.paddr = 0x40000;
        // write past the end
        let mut e = desc.write((desc.paddr + desc.size) as usize, buffer);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(true, "UnexpectedEof Error should have been generated"),
        };
        // middle at the middle past the the end
        e = desc.write((desc.paddr + desc.size - 5) as usize, buffer);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(true, "UnexpectedEof Error should have been generated"),
        };
        // read at the start past the end
        let v: Vec<u8> = vec![0; (desc.size + 8) as usize];
        buffer = &v;
        e = desc.write(desc.paddr as usize, buffer);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(true, "UnexpectedEof Error should have been generated"),
        };
    }
    #[test]
    fn test_write_errors() {
        operate_on_file(&test_write_errors_cb, DATA);
    }
}
