//! RIO plugin that opens memory based virtual files.

use crate::plugin::*;
use crate::utils::*;
use std::io;

const METADATA: RIOPluginMetadata = RIOPluginMetadata {
    name: "Malloc",
    desc: "This plugin is used to create memory based nameless files.\
           The only mode supported by this plugin is Read-Write ( you\
           cannot open memory file as read only).",
    author: "Oddcoder",
    license: "LGPL",
    version: "0.0.1",
};
struct MallocInternal {
    data: Vec<u8>,
}
impl MallocInternal {
    fn new(size: u64) -> Self {
        MallocInternal {
            data: vec![0; size as usize],
        }
    }
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl RIOPluginOperations for MallocInternal {
    fn read(&mut self, raddr: usize, buffer: &mut [u8]) -> Result<(), IoError> {
        if self.len() < raddr + buffer.len() {
            return Err(IoError::Parse(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "BufferOverflow",
            )));
        }
        buffer.copy_from_slice(&self.data[raddr..raddr + buffer.len()]);
        Ok(())
    }

    fn write(&mut self, raddr: usize, buf: &[u8]) -> Result<(), IoError> {
        if raddr + buf.len() > self.len() {
            return Err(IoError::Parse(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "BufferOverflow",
            )));
        }
        self.data[raddr..raddr + buf.len()].copy_from_slice(buf);
        Ok(())
    }
}

struct MallocPlugin {}

impl MallocPlugin {
    fn uri_to_size(uri: &str) -> Option<u64> {
        let n = uri.trim_start_matches("malloc://");
        if n.len() >= 2 {
            match &*n[0..2].to_lowercase() {
                "0b" => return u64::from_str_radix(&n[2..], 2).ok(),
                "0x" => return u64::from_str_radix(&n[2..], 16).ok(),
                _ => (),
            }
        }
        if n.len() > 1 && n.starts_with('0') {
            return u64::from_str_radix(&n[1..], 8).ok();
        }
        n.parse::<u64>().ok()
    }
}

impl RIOPlugin for MallocPlugin {
    fn get_metadata(&self) -> &'static RIOPluginMetadata {
        &METADATA
    }

    fn open(&mut self, uri: &str, flags: IoMode) -> Result<RIOPluginDesc, IoError> {
        if flags.contains(IoMode::COW) {
            return Err(IoError::Parse(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Can't open file with permission Copy-On-Write",
            )));
        }

        if !flags.contains(IoMode::READ) {
            return Err(IoError::Parse(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Memory based files must have read permission",
            )));
        }
        if !flags.contains(IoMode::WRITE) {
            return Err(IoError::Parse(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Memory based files must have write permission",
            )));
        }
        let file = match MallocPlugin::uri_to_size(uri) {
            Some(size) => MallocInternal::new(size),
            None => {
                return Err(IoError::Custom(
                    "Failed to parse given uri as usize".to_string(),
                ))
            }
        };
        let desc = RIOPluginDesc {
            name: uri.to_owned(),
            perm: flags,
            raddr: 0,
            size: (file.len() as u64),
            plugin_operations: Box::new(file),
        };
        Ok(desc)
    }

    fn accept_uri(&self, uri: &str) -> bool {
        let split: Vec<&str> = uri.split("://").collect();
        split.len() == 2 && split[0] == "malloc"
    }
}

pub fn plugin() -> Box<dyn RIOPlugin + Sync + Send> {
    Box::new(MallocPlugin {})
}

#[cfg(test)]

mod test_malloc {
    use super::*;
    #[test]
    fn test_malloc() {
        let mut p = plugin();
        let mut file = p
            .open("malloc://0x500", IoMode::READ | IoMode::WRITE)
            .unwrap();
        assert_eq!(file.size, 0x500);
        let mut buffer = [1; 100];
        file.plugin_operations.read(0x0, &mut buffer).unwrap();
        assert_eq!(&buffer[..], &[0; 100][..]);
        file.plugin_operations.write(0x0, &[0xab; 0x100]).unwrap();
        file.plugin_operations.read(0x0, &mut buffer).unwrap();
        assert_eq!(&buffer[..], &[0xab; 100][..]);
        p.open("malloc://0b100", IoMode::READ | IoMode::WRITE)
            .unwrap();
        p.open("malloc://0500", IoMode::READ | IoMode::WRITE)
            .unwrap();
        p.open("malloc://500", IoMode::READ | IoMode::WRITE)
            .unwrap();
    }

    #[test]
    fn test_malloc_errors() {
        let mut p = plugin();
        let mut err = p
            .open("malloc://0x", IoMode::READ | IoMode::WRITE)
            .err()
            .unwrap();
        assert_eq!(
            err,
            IoError::Custom("Failed to parse given uri as usize".to_string())
        );
        err = p.open("malloc://0x500", IoMode::READ).err().unwrap();
        assert_eq!(
            err,
            IoError::Parse(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Memory based files must have write permission"
            ))
        );
        err = p.open("malloc://0x500", IoMode::WRITE).err().unwrap();
        assert_eq!(
            err,
            IoError::Parse(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Memory based files must have read permission"
            ))
        );
        err = p
            .open("malloc://0x500", IoMode::READ | IoMode::WRITE | IoMode::COW)
            .err()
            .unwrap();
        assert_eq!(
            err,
            IoError::Parse(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Can't open file with permission Copy-On-Write"
            ))
        );
    }

    #[test]
    fn test_read_write_error() {
        let mut p = plugin();
        let mut file = p
            .open("malloc://0x50", IoMode::READ | IoMode::WRITE)
            .unwrap();
        assert_eq!(file.size, 0x50);
        let mut buffer = [1; 0x51];
        let mut err = file.plugin_operations.read(0, &mut buffer).err().unwrap();
        assert_eq!(
            err,
            IoError::Parse(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "BufferOverflow"
            ))
        );
        err = file.plugin_operations.write(0, &buffer).err().unwrap();
        assert_eq!(
            err,
            IoError::Parse(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "BufferOverflow"
            ))
        );
    }
}
