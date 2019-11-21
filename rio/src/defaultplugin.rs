/*
 * defaultplugin.rs: RIO plugin that opens raw binary files.
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
use memmap::*;
use plugin::*;
use std::fs::OpenOptions;
use std::io;
use std::ops::Deref;
use std::path::Path;
use utils::*;
enum FileInternals {
    Map(Mmap),
    MutMap(MmapMut),
}

impl FileInternals {
    fn len(&self) -> usize {
        match self {
            FileInternals::Map(m) => return m.len(),
            FileInternals::MutMap(m) => return m.len(),
        }
    }
    fn as_mut(&mut self) -> Option<&mut MmapMut> {
        if let FileInternals::MutMap(mutmap) = self {
            return Some(mutmap);
        } else {
            return None;
        }
    }
}
const METADATA: RIOPluginMetadata = RIOPluginMetadata {
    name: "FilePlugin",
    desc: "This IO plugin is used to open normal files.",
    author: "Oddcoder",
    license: "LGPL",
    version: "0.0.1",
};
impl Deref for FileInternals {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self {
            FileInternals::Map(m) => &**m,
            FileInternals::MutMap(m) => &**m,
        }
    }
}
impl RIOPluginOperations for FileInternals {
    fn read(&mut self, raddr: usize, buffer: &mut [u8]) -> Result<(), IoError> {
        if self.len() < raddr + buffer.len() {
            return Err(IoError::Parse(io::Error::new(io::ErrorKind::UnexpectedEof, "BufferOverflow")));
        }
        buffer.copy_from_slice(&self[raddr..raddr + buffer.len()]);
        return Ok(());
    }

    fn write(&mut self, raddr: usize, buf: &[u8]) -> Result<(), IoError> {
        if let Some(mutmap) = self.as_mut() {
            if raddr + buf.len() > mutmap.len() {
                return Err(IoError::Parse(io::Error::new(io::ErrorKind::UnexpectedEof, "BufferOverflow")));
            }
            mutmap[raddr..raddr + buf.len()].copy_from_slice(buf);
            return Ok(());
        } else {
            return Err(IoError::Parse(io::Error::new(io::ErrorKind::PermissionDenied, "File Not Writable")));
        }
    }
}

struct FilePlugin {}

impl FilePlugin {
    fn uri_to_path(uri: &str) -> &Path {
        let path = uri.trim_start_matches("file://");
        return Path::new(path);
    }
}

impl RIOPlugin for FilePlugin {
    fn get_metadata(&self) -> &'static RIOPluginMetadata {
        return &METADATA;
    }

    fn open(&mut self, uri: &str, flags: IoMode) -> Result<RIOPluginDesc, IoError> {
        let file: FileInternals;
        if !flags.contains(IoMode::READ) && flags.contains(IoMode::WRITE) {
            return Err(IoError::Parse(io::Error::new(io::ErrorKind::PermissionDenied, "Can't Open File for writing reading")));
        }
        // we can't have write with cow bcause this mean we had writer without read or read with cow lol
        if flags.contains(IoMode::READ) && flags.contains(IoMode::COW) {
            return Err(IoError::Parse(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Can't Open File with permission as Write and Copy-On-Write",
            )));
        }
        if flags.contains(IoMode::COW) {
            let f = OpenOptions::new().read(true).open(FilePlugin::uri_to_path(uri))?;
            file = FileInternals::MutMap(unsafe { MmapOptions::new().map_copy(&f)? });
        } else if flags.contains(IoMode::WRITE) {
            let f = OpenOptions::new().read(true).write(true).open(FilePlugin::uri_to_path(uri))?;
            file = FileInternals::MutMap(unsafe { MmapOptions::new().map_mut(&f)? });
        } else {
            let f = OpenOptions::new().read(true).open(FilePlugin::uri_to_path(uri))?;
            file = FileInternals::Map(unsafe { MmapOptions::new().map(&f)? });
        }
        let desc = RIOPluginDesc {
            name: uri.to_owned(),
            perm: flags,
            raddr: 0,
            size: (file.len() as u64),
            plugin_operations: Box::new(file),
        };
        return Ok(desc);
    }

    // either file:// or just no "://" to start with
    fn accept_uri(&self, uri: &str) -> bool {
        let split: Vec<&str> = uri.split("://").collect();
        if split.len() == 1 {
            return true;
        }
        if split[0] == "file" {
            return true;
        }
        return false;
    }
}

pub fn plugin() -> Box<dyn RIOPlugin> {
    return Box::new(FilePlugin {});
}

#[cfg(test)]
mod default_plugin_tests {
    use super::*;
    use test_file::*;
    #[test]
    fn test_plugin() {
        let plugin = plugin();
        let meta = plugin.get_metadata();
        assert_eq!(plugin.accept_uri("/bin/ls"), true);
        assert_eq!(plugin.accept_uri("file:///bin/ls"), true);
        assert_eq!(plugin.accept_uri("ihex:///bin/ls"), false);
        assert_eq!(meta.name, METADATA.name);
        assert_eq!(meta.desc, METADATA.desc);
        assert_eq!(meta.author, METADATA.author);
        assert_eq!(meta.license, METADATA.license);
        assert_eq!(meta.version, METADATA.version);
    }

    fn test_open_errors_cb(paths: &[&Path]) {
        let mut plugin = plugin();
        let custom_path = String::from("file://") + &paths[0].to_string_lossy();
        plugin.open(&custom_path, IoMode::COW).unwrap();
        plugin.open(&paths[1].to_string_lossy(), IoMode::READ).unwrap();
        plugin.open(&paths[2].to_string_lossy(), IoMode::READ | IoMode::WRITE).unwrap();
        let mut e = plugin.open(&paths[3].to_string_lossy(), IoMode::WRITE);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::PermissionDenied),
            _ => assert!(false, "Permission Denied Error should have been generated"),
        };

        e = plugin.open(&paths[3].to_string_lossy(), IoMode::READ | IoMode::COW);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::PermissionDenied),
            _ => assert!(false, "Permission Denied Error should have been generated"),
        };

        e = plugin.open(&paths[3].to_string_lossy(), IoMode::READ | IoMode::WRITE | IoMode::COW);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::PermissionDenied),
            _ => assert!(false, "Permission Denied Error should have been generated"),
        };
    }
    #[test]
    fn test_open_errors() {
        operate_on_files(&test_open_errors_cb, &[DATA, DATA, DATA, DATA]);
    }
    fn test_read_cb(path: &Path) {
        let mut plugin = plugin();
        let mut desc = plugin.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        let mut buffer: &mut [u8] = &mut [0; 8];
        // read at the begining
        desc.plugin_operations.read(desc.raddr as usize, &mut buffer).unwrap();
        assert_eq!(buffer, [0x00, 0x01, 0x01, 0x02, 0x03, 0x05, 0x08, 0x0d]);
        // read at the middle
        desc.plugin_operations.read((desc.raddr + 0x10) as usize, &mut buffer).unwrap();
        assert_eq!(buffer, [0xdb, 0x3d, 0x18, 0x55, 0x6d, 0xc2, 0x2f, 0xf1]);
        // read at the end
        desc.plugin_operations.read((desc.raddr + 97) as usize, &mut buffer).unwrap();
        assert_eq!(buffer, [0x41, 0xc1, 0x02, 0xc3, 0xc5, 0x88, 0x4d, 0xd5]);
    }
    #[test]
    fn test_read() {
        operate_on_file(&test_read_cb, DATA)
    }

    fn test_read_errors_cb(path: &Path) {
        let mut plugin = plugin();
        let mut desc = plugin.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        let mut buffer: &mut [u8] = &mut [0; 8];
        // read past the end
        let mut e = desc.plugin_operations.read((desc.raddr + desc.size) as usize, &mut buffer);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(true, "UnexpectedEof Error should have been generated"),
        };
        // read at the middle past the the end
        e = desc.plugin_operations.read((desc.raddr + desc.size - 5) as usize, &mut buffer);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(true, "UnexpectedEof Error should have been generated"),
        };

        // read at the start past the end
        let mut v: Vec<u8> = vec![0; (desc.size + 8) as usize];
        buffer = &mut v;
        e = desc.plugin_operations.read(desc.raddr as usize, &mut buffer);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(true, "UnexpectedEof Error should have been generated"),
        };
    }
    #[test]
    fn test_read_errors() {
        operate_on_file(&test_read_errors_cb, DATA);
    }

    fn test_write_cb(path: &Path) {
        let mut plugin = plugin();
        let mut desc = plugin.open(&path.to_string_lossy(), IoMode::READ | IoMode::WRITE).unwrap();
        let mut buffer: &mut [u8] = &mut [0; 8];
        // write at the begining
        desc.plugin_operations.write(desc.raddr as usize, &buffer).unwrap();
        desc.plugin_operations.read(desc.raddr as usize, &mut buffer).unwrap();
        assert_eq!(buffer, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // write at the middle
        desc.plugin_operations.write((desc.raddr + 0x10) as usize, &buffer).unwrap();
        desc.plugin_operations.read((desc.raddr + 0x10) as usize, &mut buffer).unwrap();
        assert_eq!(buffer, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // write at the end
        desc.plugin_operations.write((desc.raddr + 97) as usize, &buffer).unwrap();
        desc.plugin_operations.read((desc.raddr + 97) as usize, &mut buffer).unwrap();
        assert_eq!(buffer, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_write() {
        operate_on_file(&test_write_cb, DATA);
    }

    fn test_write_errors_cb(path: &Path) {
        let mut plugin = plugin();
        let mut desc = plugin.open(&path.to_string_lossy(), IoMode::READ | IoMode::WRITE).unwrap();
        let mut buffer: &[u8] = &[0; 8];
        // write past the end
        let mut e = desc.plugin_operations.write((desc.raddr + desc.size) as usize, &buffer);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(true, "UnexpectedEof Error should have been generated"),
        };
        // middle at the middle past the the end
        e = desc.plugin_operations.write((desc.raddr + desc.size - 5) as usize, &buffer);
        match e {
            Err(IoError::Parse(io_err)) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(true, "UnexpectedEof Error should have been generated"),
        };
        // read at the start past the end
        let v: Vec<u8> = vec![0; (desc.size + 8) as usize];
        buffer = &v;
        e = desc.plugin_operations.write(desc.raddr as usize, &mut buffer);
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
