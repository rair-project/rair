/*
 * ihex.rs: RIO plugin that opens memory based virtual files.
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
use plugin::*;
use utils::*;
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
struct MallocInternal{
    data: Vec<u8>,
}
impl MallocInternal {
    fn new(size: u64) -> Self{
        MallocInternal{
            data: vec![0; size as usize],
        }
    }
    fn len(&self) -> usize {
        return self.data.len();
    }
}

impl RIOPluginOperations for MallocInternal {
    fn read(&mut self, raddr: usize, buffer: &mut [u8]) -> Result<(), IoError> {
        if self.len() < raddr + buffer.len() {
            return Err(IoError::Parse(io::Error::new(io::ErrorKind::UnexpectedEof, "BufferOverflow")));
        }
        buffer.copy_from_slice(&self.data[raddr..raddr + buffer.len()]);
        return Ok(());
    }

    fn write(&mut self, raddr: usize, buf: &[u8]) -> Result<(), IoError> {
        if raddr + buf.len() > self.len() {
            return Err(IoError::Parse(io::Error::new(io::ErrorKind::UnexpectedEof, "BufferOverflow")));
        }
        self.data[raddr..raddr + buf.len()].copy_from_slice(buf);
        return Ok(());
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
        if n.len() > 1 && n.chars().nth(0).unwrap() == '0' {
            return u64::from_str_radix(&n[1..], 8).ok();
        }
        return u64::from_str_radix(n, 10).ok();
    
    }
}

impl RIOPlugin for MallocPlugin {
    fn get_metadata(&self) -> &'static RIOPluginMetadata {
        return &METADATA;
    }

    fn open(&mut self, uri: &str, flags: IoMode) -> Result<RIOPluginDesc, IoError> {
        let file: MallocInternal;
        if flags.contains(IoMode::COW) {
            return Err(IoError::Parse(io::Error::new(
                io::ErrorKind::PermissionDenied, "Can't Open File with permission Copy-On-Write",)));
        }
        
        if !flags.contains(IoMode::READ) {
            return Err(IoError::Parse(io::Error::new(io::ErrorKind::PermissionDenied, "Memory based files must have read permission")));
        }
        if !flags.contains(IoMode::WRITE) {
            return Err(IoError::Parse(io::Error::new(io::ErrorKind::PermissionDenied, "Memory based files must have write permission")));
        }
        match MallocPlugin::uri_to_size(uri) {
            Some(size) => file = MallocInternal::new(size),
            None => return Err(IoError::Custom("Failed to parse given uri as usize".to_string())),

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
        if split.len() == 2 && split[0] == "malloc" {
            return true;
        }
        return false;
    }
}

pub fn plugin() -> Box<dyn RIOPlugin> {
    return Box::new(MallocPlugin{});
}
