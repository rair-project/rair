//! RIO plugin that opens base64 encoded files.

use super::defaultplugin;
use crate::plugin::*;
use crate::utils::*;
use base64;
use base64::{decode_config_slice, encode_config_slice};
use std::cmp;
use std::io;
use std::path::Path;
const METADATA: RIOPluginMetadata = RIOPluginMetadata {
    name: "Base64",
    desc: "This plugin is used to open base64 encoded files.",
    author: "Oddcoder",
    license: "LGPL",
    version: "0.0.1",
};
struct Base64Internal {
    file: Box<dyn RIOPluginOperations + Sync + Send>, // defaultplugin
    len: u64,
}
impl Base64Internal {
    fn len(&self) -> u64 {
        self.len
    }
    fn read_first_unaligned_block<'a>(
        &mut self,
        raddr: usize,
        buffer: &'a mut [u8],
    ) -> Result<(usize, &'a mut [u8]), IoError> {
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
        Ok((raddr + size, &mut buffer[size..]))
    }

    fn read_last_unaligned_block<'a>(
        &mut self,
        raddr: usize,
        buffer: &'a mut [u8],
    ) -> Result<(usize, &'a mut [u8]), IoError> {
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
        Ok((raddr, &mut buffer[..offset]))
    }
    // returns new raddr and new buffer to fill;
    fn read_unaligned_blocks<'a>(
        &mut self,
        raddr: usize,
        buffer: &'a mut [u8],
    ) -> Result<(usize, &'a mut [u8]), IoError> {
        let (raddr, buffer) = self.read_first_unaligned_block(raddr, buffer)?;
        let (raddr, buffer) = self.read_last_unaligned_block(raddr, buffer)?;
        Ok((raddr, buffer))
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
        Ok(())
    }
    fn write_first_unaligned_block<'a>(
        &mut self,
        raddr: usize,
        buffer: &'a [u8],
    ) -> Result<(usize, &'a [u8]), IoError> {
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
        encode_config_slice(&decoded_data, base64::STANDARD, &mut b64data);
        self.file.write(b64base, &b64data)?;
        Ok((raddr + size, &buffer[size..]))
    }

    fn write_last_unaligned_block<'a>(
        &mut self,
        raddr: usize,
        buffer: &'a [u8],
    ) -> Result<(usize, &'a [u8]), IoError> {
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
        encode_config_slice(&decoded_data, base64::STANDARD, &mut b64data);
        self.file.write(b64base, &b64data)?;
        Ok((raddr, &buffer[..offset]))
    }

    fn write_unaligned_blocks<'a>(
        &mut self,
        raddr: usize,
        buffer: &'a [u8],
    ) -> Result<(usize, &'a [u8]), IoError> {
        let (raddr, buffer) = self.write_first_unaligned_block(raddr, buffer)?;
        let (raddr, buffer) = self.write_last_unaligned_block(raddr, buffer)?;
        Ok((raddr, buffer))
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
        Ok(())
    }
}

impl RIOPluginOperations for Base64Internal {
    fn read(&mut self, raddr: usize, buffer: &mut [u8]) -> Result<(), IoError> {
        if (self.len() as usize) < raddr + buffer.len() {
            return Err(IoError::Parse(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "BufferOverflow",
            )));
        }
        let (raddr, buffer) = self.read_unaligned_blocks(raddr, buffer)?;
        self.read_aligned_blocks(raddr, buffer)?;
        Ok(())
    }

    fn write(&mut self, raddr: usize, buffer: &[u8]) -> Result<(), IoError> {
        if raddr + buffer.len() > (self.len() as usize) {
            return Err(IoError::Parse(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "BufferOverflow",
            )));
        }
        let (raddr, buffer) = self.write_unaligned_blocks(raddr, buffer)?;
        self.write_aligned_blocks(raddr, buffer)?;
        Ok(())
    }
}

struct Base64Plugin {
    defaultplugin: Box<dyn RIOPlugin + Sync + Send>, // defaultplugin
}

impl Base64Plugin {
    fn uri_to_path(uri: &str) -> &Path {
        let path = uri.trim_start_matches("b64://");
        Path::new(path)
    }
    fn new() -> Base64Plugin {
        Base64Plugin {
            defaultplugin: defaultplugin::plugin(),
        }
    }
}

impl RIOPlugin for Base64Plugin {
    fn get_metadata(&self) -> &'static RIOPluginMetadata {
        &METADATA
    }

    fn open(&mut self, uri: &str, flags: IoMode) -> Result<RIOPluginDesc, IoError> {
        let mut def_desc = self
            .defaultplugin
            .open(&Base64Plugin::uri_to_path(uri).to_string_lossy(), flags)?;
        let mut paddings = [0; 4];
        if def_desc
            .plugin_operations
            .read(def_desc.size as usize - paddings.len(), &mut paddings)
            .is_err()
        {
            return Err(IoError::Custom("Corrupted base64 data".to_string()));
        };
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
        Ok(desc)
    }

    fn accept_uri(&self, uri: &str) -> bool {
        let split: Vec<&str> = uri.split("://").collect();
        split.len() == 2 && split[0] == "b64"
    }
}

pub fn plugin() -> Box<dyn RIOPlugin + Sync + Send> {
    Box::new(Base64Plugin::new())
}

#[cfg(test)]
mod test_base64 {
    use super::*;
    use test_file::*;

    #[test]
    fn test_nopad_read() {
        let mut p = plugin();
        let mut file = p
            .open(
                "b64://../../testing_binaries/rio/base64/no_padding.b64",
                IoMode::READ,
            )
            .unwrap();
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

    #[test]
    fn test_one_pad_read() {
        let mut p = plugin();
        let mut file = p
            .open(
                "b64://../../testing_binaries/rio/base64/one_pad.b64",
                IoMode::READ,
            )
            .unwrap();
        assert_eq!(file.size, 2);
        let mut b1 = [0; 1];
        file.plugin_operations.read(0, &mut b1).unwrap();
        assert_eq!(b1, [b'T']);
        file.plugin_operations.read(1, &mut b1).unwrap();
        assert_eq!(b1, [b'h']);
        let mut b2 = [0; 2];
        file.plugin_operations.read(0, &mut b2).unwrap();
        assert_eq!(b2, [b'T', b'h']);
        let e = file.plugin_operations.read(1, &mut b2).err().unwrap();
        assert_eq!(
            e,
            IoError::Parse(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "BufferOverflow"
            ))
        );
    }

    #[test]
    fn test_two_pad_read() {
        let mut p = plugin();
        let mut file = p
            .open(
                "b64://../../testing_binaries/rio/base64/two_pad.b64",
                IoMode::READ,
            )
            .unwrap();
        assert_eq!(file.size, 1);
        let mut b1 = [0; 1];
        file.plugin_operations.read(0, &mut b1).unwrap();
        assert_eq!(b1, [b'T']);
        let e = file.plugin_operations.read(1, &mut b1).err().unwrap();
        assert_eq!(
            e,
            IoError::Parse(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "BufferOverflow"
            ))
        );
    }
    fn nopad_write_cb(path: &Path) {
        let mut p = plugin();
        let uri = String::from("b64://") + &path.to_string_lossy();
        let mut file = p.open(&uri, IoMode::READ | IoMode::WRITE).unwrap();
        file.plugin_operations.write(0, b"t").unwrap();
        file.plugin_operations.write(1, b"HE").unwrap();
        file.plugin_operations.write(3, b"_QU").unwrap();
        file.plugin_operations.write(6, b"ICK_").unwrap();
        file.plugin_operations.write(10, b"BROWN").unwrap();
        file.plugin_operations.write(15, b"_FOX_J").unwrap();
        file.plugin_operations.write(21, b"UMPED_O").unwrap();
        file.plugin_operations.write(28, b"VER_THE_").unwrap();
        file.plugin_operations.write(36, b"LAZY_DOG;").unwrap();
        let mut data = [0; 45];
        file.plugin_operations.read(0, &mut data).unwrap();
        assert_eq!(
            &data[..],
            &b"tHE_QUICK_BROWN_FOX_JUMPED_OVER_THE_LAZY_DOG;"[..]
        );
    }
    #[test]
    fn test_nopad_write() {
        operate_on_copy(
            &nopad_write_cb,
            "../../testing_binaries/rio/base64/no_padding.b64",
        );
    }

    fn one_pad_write_cb(path: &Path) {
        let mut p = plugin();
        let uri = String::from("b64://") + &path.to_string_lossy();
        let mut file = p.open(&uri, IoMode::READ | IoMode::WRITE).unwrap();
        file.plugin_operations.write(0, b"t").unwrap();
        file.plugin_operations.write(1, b"H").unwrap();
        let mut d2 = [0; 2];
        file.plugin_operations.read(0, &mut d2).unwrap();
        assert_eq!(&d2[..], &b"tH"[..]);
        file.plugin_operations.write(0, b"Th").unwrap();
        file.plugin_operations.read(0, &mut d2).unwrap();
        assert_eq!(&d2[..], &b"Th"[..]);
        let e = file.plugin_operations.write(1, &mut d2).err().unwrap();
        assert_eq!(
            e,
            IoError::Parse(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "BufferOverflow"
            ))
        );
    }

    #[test]
    fn test_one_pad_write() {
        operate_on_copy(
            &one_pad_write_cb,
            "../../testing_binaries/rio/base64/one_pad.b64",
        );
    }

    fn two_pad_write_cb(path: &Path) {
        let mut p = plugin();
        let uri = String::from("b64://") + &path.to_string_lossy();
        let mut file = p.open(&uri, IoMode::READ | IoMode::WRITE).unwrap();
        file.plugin_operations.write(0, b"t").unwrap();
        let mut d2 = [0; 1];
        file.plugin_operations.read(0, &mut d2).unwrap();
        assert_eq!(&d2[..], &b"t"[..]);
        file.plugin_operations.write(0, b"T").unwrap();
        file.plugin_operations.read(0, &mut d2).unwrap();
        assert_eq!(&d2[..], &b"T"[..]);
        let e = file.plugin_operations.write(1, b"H").err().unwrap();
        assert_eq!(
            e,
            IoError::Parse(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "BufferOverflow"
            ))
        );
    }
    #[test]
    fn test_two_pad_write() {
        operate_on_copy(
            &two_pad_write_cb,
            "../../testing_binaries/rio/base64/two_pad.b64",
        );
    }
}
