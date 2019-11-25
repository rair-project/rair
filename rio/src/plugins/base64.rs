/*
 * base64.rs: RIO plugin that opens base64 encoded files.
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
use super::defaultplugin;
use base64;
use base64::{decode_config_slice, encode_config_slice};
use plugin::*;
use std::cmp;
use std::io;
use std::path::Path;
use utils::*;
const METADATA: RIOPluginMetadata = RIOPluginMetadata {
    name: "Base64",
    desc: "This plugin is used to open base64 encoded files.",
    author: "Oddcoder",
    license: "LGPL",
    version: "0.0.1",
};
struct Base64Internal {
    file: Box<dyn RIOPluginOperations>, // defaultplugin
    len: u64,
}
impl Base64Internal {
    fn len(&self) -> u64 {
        return self.len;
    }
    fn read_first_unaligned_block<'a>(&mut self, raddr: usize, buffer: &'a mut [u8]) -> Result<(usize, &'a mut [u8]), IoError> {
        let offset = raddr % 3;
        if offset == 0 {
            return Ok((raddr, buffer));
        }
        let base = raddr - offset;
        let size = cmp::min(3 - offset, buffer.len());
        let b64base = base / 3 * 4;
        let mut b64data = [0; 4];
        let mut decoded_data = [0; 3];
        self.file.read(b64base, &mut b64data)?;
        if decode_config_slice(&b64data, base64::STANDARD, &mut decoded_data).is_err() {
            return Err(IoError::Custom("Corrupted base64 data".to_string()));
        }
        buffer[0..size].copy_from_slice(&decoded_data[offset..offset + size]);
        return Ok((raddr + size, &mut buffer[size..]));
    }

    fn read_last_unaligned_block<'a>(&mut self, raddr: usize, buffer: &'a mut [u8]) -> Result<(usize, &'a mut [u8]), IoError> {
        // we assume that raddr is always aligned on the start
        let size = buffer.len() % 3;
        if size == 0 {
            return Ok((raddr, buffer));
        }
        let offset = buffer.len() - size;
        let base = raddr + offset;
        let b64base = base / 3 * 4;
        let mut b64data = [0; 4];
        let mut decoded_data = [0; 3];
        self.file.read(b64base, &mut b64data)?;
        if decode_config_slice(&b64data, base64::STANDARD, &mut decoded_data).is_err() {
            return Err(IoError::Custom("Corrupted base64 data".to_string()));
        }
        buffer[offset..].copy_from_slice(&decoded_data[0..size]);
        return Ok((raddr, &mut buffer[..offset]));
    }
    // returns new raddr and new buffer to fill;
    fn read_unaligned_blocks<'a>(&mut self, raddr: usize, buffer: &'a mut [u8]) -> Result<(usize, &'a mut [u8]), IoError> {
        let (raddr, buffer) = self.read_first_unaligned_block(raddr, buffer)?;
        let (raddr, buffer) = self.read_last_unaligned_block(raddr, buffer)?;
        return Ok((raddr, buffer));
    }
    fn read_aligned_blocks(&mut self, raddr: usize, buffer: &mut [u8]) -> Result<(), IoError> {
        if buffer.is_empty() {
            return Ok(());
        }
        let b64size = buffer.len() / 3 * 4;
        let b64addr = raddr / 3 * 4;
        let mut b64data = vec![0; b64size];
        self.file.read(b64addr, &mut b64data)?;
        if decode_config_slice(&b64data, base64::STANDARD, buffer).is_err() {
            return Err(IoError::Custom("Corrupted base64 data".to_string()));
        }
        return Ok(());
    }
    fn write_first_unaligned_block<'a>(&mut self, raddr: usize, buffer: &'a [u8]) -> Result<(usize, &'a [u8]), IoError> {
        let offset = raddr % 3;
        if offset == 0 {
            return Ok((raddr, buffer));
        }

        let base = raddr - offset;
        let size = cmp::min(3 - offset, buffer.len());
        let b64base = base / 3 * 4;
        let mut b64data = [0; 4];
        let mut decoded_data = [0; 3];
        self.file.read(b64base, &mut b64data)?;
        if decode_config_slice(&b64data, base64::STANDARD, &mut decoded_data).is_err() {
            return Err(IoError::Custom("Corrupted base64 data".to_string()));
        }
        decoded_data[offset..offset + size].copy_from_slice(&buffer[0..size]);
        encode_config_slice(&decoded_data[0..offset + size], base64::STANDARD, &mut b64data);
        self.file.write(b64base, &b64data)?;
        return Ok((raddr + size, &buffer[size..]));
    }

    fn write_last_unaligned_block<'a>(&mut self, raddr: usize, buffer: &'a [u8]) -> Result<(usize, &'a [u8]), IoError> {
        // we assume that raddr is always aligned on the start
        let size = buffer.len() % 3;
        if size == 0 {
            return Ok((raddr, buffer));
        }
        let offset = buffer.len() - size;
        let base = raddr + offset;
        let b64base = base / 3 * 4;
        let mut b64data = [0; 4];
        let mut decoded_data = [0; 3];
        self.file.read(b64base, &mut b64data)?;
        if decode_config_slice(&b64data, base64::STANDARD, &mut decoded_data).is_err() {
            return Err(IoError::Custom("Corrupted base64 data".to_string()));
        }
        decoded_data[0..size].copy_from_slice(&buffer[offset..]);
        encode_config_slice(&decoded_data[0..offset + size], base64::STANDARD, &mut b64data);
        self.file.write(b64base, &b64data)?;
        return Ok((raddr, &buffer[..offset]));
    }

    fn write_unaligned_blocks<'a>(&mut self, raddr: usize, buffer: &'a [u8]) -> Result<(usize, &'a [u8]), IoError> {
        let (raddr, buffer) = self.write_first_unaligned_block(raddr, buffer)?;
        let (raddr, buffer) = self.write_last_unaligned_block(raddr, buffer)?;
        return Ok((raddr, buffer));
    }
    fn write_aligned_blocks(&mut self, raddr: usize, buffer: &[u8]) -> Result<(), IoError> {
        if buffer.is_empty() {
            return Ok(());
        }
        let b64size = buffer.len() / 3 * 4;
        let b64addr = raddr / 3 * 4;
        let mut b64data = vec![0; b64size];
        encode_config_slice(buffer, base64::STANDARD, &mut b64data);
        self.file.write(b64addr, &b64data)?;
        return Ok(());
    }
}

impl RIOPluginOperations for Base64Internal {
    fn read(&mut self, raddr: usize, buffer: &mut [u8]) -> Result<(), IoError> {
        if (self.len() as usize) < raddr + buffer.len() {
            return Err(IoError::Parse(io::Error::new(io::ErrorKind::UnexpectedEof, "BufferOverflow")));
        }
        let (raddr, buffer) = self.read_unaligned_blocks(raddr, buffer)?;
        self.read_aligned_blocks(raddr, buffer)?;
        return Ok(());
    }

    fn write(&mut self, raddr: usize, buffer: &[u8]) -> Result<(), IoError> {
        if raddr + buffer.len() > (self.len() as usize) {
            return Err(IoError::Parse(io::Error::new(io::ErrorKind::UnexpectedEof, "BufferOverflow")));
        }
        let (raddr, buffer) = self.write_unaligned_blocks(raddr, buffer)?;
        self.write_aligned_blocks(raddr, buffer)?;
        return Ok(());
    }
}

struct Base64Plugin {
    defaultplugin: Box<dyn RIOPlugin>, // defaultplugin
}

impl Base64Plugin {
    fn uri_to_path(uri: &str) -> &Path {
        let path = uri.trim_start_matches("b64://");
        return Path::new(path);
    }
    fn new() -> Base64Plugin {
        Base64Plugin {
            defaultplugin: defaultplugin::plugin(),
        }
    }
}

impl RIOPlugin for Base64Plugin {
    fn get_metadata(&self) -> &'static RIOPluginMetadata {
        return &METADATA;
    }

    fn open(&mut self, uri: &str, flags: IoMode) -> Result<RIOPluginDesc, IoError> {
        let mut def_desc = self.defaultplugin.open(&Base64Plugin::uri_to_path(uri).to_string_lossy(), flags)?;
        let mut paddings = [0; 2];
        def_desc.plugin_operations.read(def_desc.size as usize - 2, &mut paddings).unwrap();
        let padding_size = paddings.iter().filter(|&n| *n == b'=').count();
        let internal = Base64Internal {
            file: def_desc.plugin_operations,
            // each 1, 2, or 3 bytes are mapped to 4 bytes
            len: def_desc.size / 4 * 3 - padding_size as u64,
        };
        let desc = RIOPluginDesc {
            name: uri.to_owned(),
            perm: flags,
            raddr: 0,
            size: internal.len(),
            plugin_operations: Box::new(internal),
        };
        return Ok(desc);
    }

    fn accept_uri(&self, uri: &str) -> bool {
        let split: Vec<&str> = uri.split("://").collect();
        if split.len() == 2 && split[0] == "b64" {
            return true;
        }
        return false;
    }
}

pub fn plugin() -> Box<dyn RIOPlugin> {
    return Box::new(Base64Plugin::new());
}

#[cfg(test)]
mod test_base64 {
    use super::*;
    #[test]
    fn test_nopad_read() {
        let mut p = plugin();
        let mut file = p.open("b64://../../testing_binaries/rio/base64/no_padding.b64", IoMode::READ).unwrap();
        assert_eq!(file.size, 45);
        let bytes = b"The quick brown fox jumped over the lazy dog.";
        //read from the start
        for i in 1..20 {
            let mut buffer = vec![0; i];
            for j in 0..20 {
                file.plugin_operations.read(j, &mut buffer).unwrap();
                assert_eq!(buffer, &bytes[j..j + i]);
            }
        }
        let mut buffer = vec![0; 45];
        file.plugin_operations.read(0, &mut buffer).unwrap();
        assert_eq!(buffer, &bytes[..]);
    }
}
