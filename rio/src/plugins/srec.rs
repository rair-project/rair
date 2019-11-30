/*
 * srec.rs: RIO plugin that opens  Motorola S-records files.
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
use super::dummy::Dummy;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while_m_n;
use nom::combinator::map_res;
use nom::sequence::tuple;
use nom::IResult;
use plugin::*;
use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write as IoWrite;
use std::path::Path;
use std::str;
use utils::*;

const METADATA: RIOPluginMetadata = RIOPluginMetadata {
    name: "Srec",
    desc: "This IO plugin is used to open Motorola \
           S-records(srec) files, this plugin would fill \
           sparce Motorola srec files with zeros when doing \
           read operation but in case of writes, unfilled \
           bytes will remain unfilled",
    author: "Oddcoder",
    license: "LGPL",
    version: "0.0.1",
};
struct SrecInternal {
    file: Box<dyn RIOPluginOperations>, // defaultplugin
    bytes: BTreeMap<u64, u8>,           // sparce array of bytes
    uri: String,
    prot: IoMode,
    start_address: Option<u64>, // I am not sure if this will always exist or not
    header: Vec<u8>,
}
enum Record {
    Header(Vec<u8>),    // Record S0 (header data)
    Data(u64, Vec<u8>), // Record S1, S2, S3  (base address, bytes)
    Count(u64),         // s5, s6
    EOF(u64),           // S7, s8, s9 (start address)
}

named!(parse_newline, alt!(tag!("\r\n") | tag!("\n") | tag!("\r")));

fn from_hex(input: &[u8]) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(str::from_utf8(input).unwrap(), 16)
}

fn is_hex_digit(c: u8) -> bool {
    (c as char).is_digit(16)
}

fn hex_byte(input: &[u8]) -> IResult<&[u8], u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

fn hex_big_word(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, (byte1, byte2)) = tuple((hex_byte, hex_byte))(input)?;
    let result = ((byte1 as u16) << 8) + byte2 as u16;
    return Ok((input, result));
}
fn hex_big_24bits(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, (byte, word)) = tuple((hex_byte, hex_big_word))(input)?;
    let result = ((byte as u32) << 16) + word as u32;
    return Ok((input, result));
}
fn hex_big_dword(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, (word1, word2)) = tuple((hex_big_word, hex_big_word))(input)?;
    let result = ((word1 as u32) << 16) + word2 as u32;
    return Ok((input, result));
}

fn parse_record0(input: &[u8]) -> IResult<&[u8], Record> {
    let (input, _) = tag("S0")(input)?;
    let (input, total_size) = hex_byte(input)?;
    let size = total_size - 3; // 2 bytes for the address, 1 byte for the checksum
    let (mut input, _) = hex_big_word(input)?;
    let mut data = Vec::with_capacity(size as usize);
    for _ in 0..size {
        let x = hex_byte(input)?;
        input = x.0;
        data.push(x.1);
    }
    let (input, _) = hex_byte(input)?; //checksum
    let (input, _) = parse_newline(input)?; //newline
    return Ok((input, Record::Header(data)));
}
fn parse_record1(input: &[u8]) -> IResult<&[u8], Record> {
    let (input, _) = tag("S1")(input)?;
    let (input, total_size) = hex_byte(input)?;
    let size = total_size - 3; // 16 bits for the address, 1 byte for the checksum
    let (mut input, addr) = hex_big_word(input)?;
    let mut data = Vec::with_capacity(size as usize);
    for _ in 0..size {
        let x = hex_byte(input)?;
        input = x.0;
        data.push(x.1);
    }
    let (input, _) = hex_byte(input)?; //checksum
    let (input, _) = parse_newline(input)?; //newline
    return Ok((input, Record::Data(addr as u64, data)));
}
fn parse_record2(input: &[u8]) -> IResult<&[u8], Record> {
    let (input, _) = tag("S2")(input)?;
    let (input, total_size) = hex_byte(input)?;
    let size = total_size - 4; // 24 bits for the address, 1 byte for the checksum
    let (mut input, addr) = hex_big_24bits(input)?;
    let mut data = Vec::with_capacity(size as usize);
    for _ in 0..size {
        let x = hex_byte(input)?;
        input = x.0;
        data.push(x.1);
    }
    let (input, _) = hex_byte(input)?; //checksum
    let (input, _) = parse_newline(input)?; //newline
    return Ok((input, Record::Data(addr as u64, data)));
}
fn parse_record3(input: &[u8]) -> IResult<&[u8], Record> {
    let (input, _) = tag("S3")(input)?;
    let (input, total_size) = hex_byte(input)?;
    let size = total_size - 5; // 32 bits for the address, 1 byte for the checksum
    let (mut input, addr) = hex_big_dword(input)?;
    let mut data = Vec::with_capacity(size as usize);
    for _ in 0..size {
        let x = hex_byte(input)?;
        input = x.0;
        data.push(x.1);
    }
    let (input, _) = hex_byte(input)?; //checksum
    let (input, _) = parse_newline(input)?; //newline
    return Ok((input, Record::Data(addr as u64, data)));
}
fn parse_record5(input: &[u8]) -> IResult<&[u8], Record> {
    let (input, _) = tag("S503")(input)?;
    let (input, count) = hex_big_word(input)?;
    let (input, _) = hex_byte(input)?; //checksum
    let (input, _) = parse_newline(input)?; //newline
    return Ok((input, Record::Count(count as u64)));
}
fn parse_record6(input: &[u8]) -> IResult<&[u8], Record> {
    let (input, _) = tag("S604")(input)?;
    let (input, count) = hex_big_24bits(input)?;
    let (input, _) = hex_byte(input)?; //checksum
    let (input, _) = parse_newline(input)?; //newline
    return Ok((input, Record::Count(count as u64)));
}
fn parse_record7(input: &[u8]) -> IResult<&[u8], Record> {
    let (input, _) = tag("S705")(input)?;
    let (input, start) = hex_big_dword(input)?;
    let (input, _) = hex_byte(input)?; //checksum
    return Ok((input, Record::EOF(start as u64)));
}
fn parse_record8(input: &[u8]) -> IResult<&[u8], Record> {
    let (input, _) = tag("S804")(input)?;
    let (input, start) = hex_big_24bits(input)?;
    let (input, _) = hex_byte(input)?; //checksum
    return Ok((input, Record::EOF(start as u64)));
}
fn parse_record9(input: &[u8]) -> IResult<&[u8], Record> {
    let (input, _) = tag("S903")(input)?;
    let (input, start) = hex_big_word(input)?;
    let (input, _) = hex_byte(input)?; //checksum
    return Ok((input, Record::EOF(start as u64)));
}
named!(parse_record(&[u8]) -> Record, alt!(parse_record0 | parse_record1 | parse_record2 | parse_record3 | parse_record5 | parse_record6 | parse_record7 | parse_record8 | parse_record9));

impl SrecInternal {
    fn parse_srec(&mut self, input: &[u8]) -> Result<(), IoError> {
        let mut input = input;
        let mut line = 1;
        loop {
            let x = match parse_record(input) {
                Ok(x) => x,
                Err(_) => return Err(IoError::Custom(format!("Invalid S-record at line: {}", line))),
            };
            input = x.0;
            match x.1 {
                Record::EOF(start) => {
                    self.start_address = Some(start);
                    break;
                }
                Record::Data(base, data) => {
                    for i in 0..data.len() as u64 {
                        self.bytes.insert(i + base, data[i as usize]);
                    }
                }
                Record::Header(header) => self.header = header,
                _ => (),
            }
            line += 1;
        }
        return Ok(());
    }
    fn size(&self) -> u64 {
        let min = if let Some((k, _)) = self.bytes.iter().next() {
            k
        } else {
            return 0;
        };
        let max = if let Some((k, _)) = self.bytes.iter().next_back() {
            k
        } else {
            return 0;
        };
        return max - min + 1;
    }
    fn base(&self) -> u64 {
        if let Some((k, _)) = self.bytes.iter().next() {
            return *k;
        } else {
            return 0;
        };
    }
    fn write_header(&mut self, file: &mut File) -> Result<(), IoError> {
        if self.header.len() > 0xff {
            return Err(IoError::Custom("Cannot write S0 Entry with size > 0xff".to_string()));
        }
        write!(file, "S0{:02x}0000", self.header.len() + 3).unwrap();
        let mut checksum = self.header.len() as u16;
        for byte in self.header.iter() {
            checksum = (checksum + *byte as u16) & 0xff;
            write!(file, "{:02x}", byte).unwrap();
        }
        writeln!(file, "{:02x}", !((checksum & 0xff) as u8)).unwrap();
        return Ok(());
    }
    fn write_data(&mut self, file: &mut File) -> Result<(), IoError> {
        let mut checksum: u16 = 0x10;
        let mut data = String::new();
        let mut record: &str = "S1";
        let mut addr = 0;
        let mut i = 0;
        let mut extra_data = 0;
        for (k, v) in self.bytes.iter() {
            if i != 0 {
                if i == 0x10 || *k != addr + 1 {
                    let size = i + extra_data;
                    checksum = (!(checksum + size)) & 0xff;
                    writeln!(file, "{}{:02x}{}{:02x}", record, size, data, checksum)?;

                    data.clear();
                    checksum = 0;
                    i = 0;
                } else {
                    // we know that *k == addr + 1
                    addr = *k;
                    write!(data, "{:02x}", *v).unwrap();
                    checksum = (checksum + *v as u16) & 0xff;
                }
            }
            if i == 0 {
                if *k > 0x00ff_ffff {
                    // record S3
                    record = "S3";
                    extra_data = 5;
                    write!(data, "{:08x}", *k).unwrap();
                } else if *k > 0xffff {
                    // record S2
                    record = "S2";
                    extra_data = 4;
                    write!(data, "{:06x}", *k).unwrap();
                } else {
                    // record S1
                    record = "S1";
                    extra_data = 3;
                    write!(data, "{:04x}", *k).unwrap();
                }
                for byte in k.to_be_bytes().iter() {
                    checksum = (checksum + *byte as u16) & 0xff;
                }
                write!(data, "{:02x}", *v).unwrap();
                checksum = (checksum + *v as u16) & 0xff;
                addr = *k;
            }
            i += 1;
        }
        if !data.is_empty() {
            let size = i + extra_data;
            checksum = (!(checksum + size)) & 0xff;
            writeln!(file, "{}{:02x}{}{:02x}", record, size, data, checksum)?;
        }

        return Ok(());
    }
    fn write_eof(&mut self, file: &mut File) -> Result<(), IoError> {
        let start = match self.start_address {
            Some(start) => start,
            None => return Ok(()),
        };
        let mut checksum: u16;
        if start > 0x00ff_ffff {
            // record S7
            write!(file, "S705{:08x}", start)?;
            checksum = 0x5;
        } else if start > 0xffff {
            //record S8
            write!(file, "S804{:06x}", start)?;
            checksum = 0x4;
        } else {
            // record S9
            write!(file, "S903{:04x}", start)?;
            checksum = 0x3;
        }
        for byte in start.to_be_bytes().iter() {
            checksum = (checksum + *byte as u16) & 0xff;
        }
        writeln!(file, "{:02x}", checksum).unwrap();
        return Ok(());
    }
    fn save_srec(&mut self) -> Result<(), IoError> {
        let mut file = OpenOptions::new().write(true).truncate(true).open(SrecPlugin::uri_to_path(&self.uri))?;
        self.write_header(&mut file)?;
        self.write_data(&mut file)?;
        self.write_eof(&mut file)?;
        return Ok(());
    }
}

impl RIOPluginOperations for SrecInternal {
    fn read(&mut self, raddr: usize, buffer: &mut [u8]) -> Result<(), IoError> {
        for (i, item) in buffer.iter_mut().enumerate() {
            let addr = (i + raddr) as u64;
            if let Some(v) = self.bytes.get(&addr) {
                *item = *v;
            } else {
                *item = 0;
            }
        }
        return Ok(());
    }

    fn write(&mut self, raddr: usize, buf: &[u8]) -> Result<(), IoError> {
        // if we are dealing with cow or write first write data to the sparce array
        if !self.prot.contains(IoMode::COW) && !self.prot.contains(IoMode::WRITE) {
            return Err(IoError::Parse(io::Error::new(io::ErrorKind::PermissionDenied, "File Not Writable")));
        }
        for (i, item) in buf.iter().enumerate() {
            self.bytes.insert((i + raddr) as u64, *item);
        }

        if self.prot.contains(IoMode::WRITE) {
            // drop old file descriptor
            self.file = Box::new(Dummy {});
            // write data to new file with old file name
            self.save_srec()?;
            // mmap new file
            let mut plug = defaultplugin::plugin();
            let def_desc = plug.open(&SrecPlugin::uri_to_path(&self.uri).to_string_lossy(), IoMode::READ)?;
            self.file = def_desc.plugin_operations;
        }
        return Ok(());
    }
}

struct SrecPlugin {
    defaultplugin: Box<dyn RIOPlugin>, // defaultplugin
}

impl SrecPlugin {
    fn uri_to_path(uri: &str) -> &Path {
        let path = uri.trim_start_matches("srec://");
        return Path::new(path);
    }
    fn new() -> Self {
        SrecPlugin {
            defaultplugin: defaultplugin::plugin(),
        }
    }
}

impl RIOPlugin for SrecPlugin {
    fn get_metadata(&self) -> &'static RIOPluginMetadata {
        return &METADATA;
    }

    fn open(&mut self, uri: &str, flags: IoMode) -> Result<RIOPluginDesc, IoError> {
        assert!(self.accept_uri(uri));
        let def_desc = self.defaultplugin.open(&SrecPlugin::uri_to_path(uri).to_string_lossy(), IoMode::READ)?;
        let mut internal = SrecInternal {
            file: def_desc.plugin_operations,
            bytes: BTreeMap::new(),
            prot: flags,
            uri: uri.to_string(),
            start_address: None,
            header: Vec::new(),
        };
        let mut data = vec![0; def_desc.size as usize];
        internal.file.read(0x0, &mut data)?;
        internal.parse_srec(&data)?;
        let desc = RIOPluginDesc {
            name: uri.to_owned(),
            perm: flags,
            raddr: internal.base(),
            size: internal.size(),
            plugin_operations: Box::new(internal),
        };
        return Ok(desc);
    }

    fn accept_uri(&self, uri: &str) -> bool {
        let split: Vec<&str> = uri.split("://").collect();
        if split.len() == 2 && split[0] == "srec" {
            return true;
        }
        return false;
    }
}

pub fn plugin() -> Box<dyn RIOPlugin> {
    return Box::new(SrecPlugin::new());
}

#[cfg(test)]
mod test_srec {
    use super::*;
    use test_file::*;
    #[test]
    fn test_record0() {
        let input = b"S021000036384B50524F47202020323043524541544544204259204541535936384B6D\n";
        let (input, rec) = parse_record0(input).unwrap();
        assert_eq!(input, b"");
        match rec {
            Record::Header(header) => assert_eq!(header, b"68KPROG   20CREATED BY EASY68K"),
            _ => panic!("Expected Header record"),
        }
    }

    #[test]
    fn test_record1() {
        let input = b"S1231000427900001142103C0020123C00004E4F123C00014E4F2841123C00024E4F2641BE\n";
        let (input, rec) = parse_record1(input).unwrap();
        assert_eq!(input, b"");
        if let Record::Data(addr, data) = rec {
            assert_eq!(addr, 0x1000);
            assert_eq!(data.len(), 0x20);
            assert_eq!(
                data,
                [
                    0x42, 0x79, 0x00, 0x00, 0x11, 0x42, 0x10, 0x3C, 0x00, 0x20, 0x12, 0x3C, 0x00, 0x00, 0x4E, 0x4F, 0x12, 0x3C, 0x00, 0x01, 0x4E, 0x4F, 0x28, 0x41, 0x12, 0x3C, 0x00, 0x02, 0x4E, 0x4F,
                    0x26, 0x41
                ]
            );
        } else {
            panic!("Expected Data record");
        }
    }
    #[test]
    fn test_record2() {
        let input = b"S2234210007900001142103C0020123C00004E4F123C00014E4F2841123C00024E4F2641BE\n";
        let (input, rec) = parse_record2(input).unwrap();
        assert_eq!(input, b"");
        if let Record::Data(addr, data) = rec {
            assert_eq!(addr, 0x421000);
            assert_eq!(data.len(), 0x1f);
            assert_eq!(
                data,
                [
                    0x79, 0x00, 0x00, 0x11, 0x42, 0x10, 0x3C, 0x00, 0x20, 0x12, 0x3C, 0x00, 0x00, 0x4E, 0x4F, 0x12, 0x3C, 0x00, 0x01, 0x4E, 0x4F, 0x28, 0x41, 0x12, 0x3C, 0x00, 0x02, 0x4E, 0x4F, 0x26,
                    0x41
                ]
            );
        } else {
            panic!("Expected Data record");
        }
    }
    #[test]
    fn test_record3() {
        let input = b"S3234200100079001142103C0020123C00004E4F123C00014E4F2841123C00024E4F2641BE\n";
        let (input, rec) = parse_record3(input).unwrap();
        assert_eq!(input, b"");
        if let Record::Data(addr, data) = rec {
            assert_eq!(addr, 0x42001000);
            assert_eq!(data.len(), 0x1e);
            assert_eq!(
                data,
                [0x79, 0x00, 0x11, 0x42, 0x10, 0x3C, 0x00, 0x20, 0x12, 0x3C, 0x00, 0x00, 0x4E, 0x4F, 0x12, 0x3C, 0x00, 0x01, 0x4E, 0x4F, 0x28, 0x41, 0x12, 0x3C, 0x00, 0x02, 0x4E, 0x4F, 0x26, 0x41]
            );
        } else {
            panic!("Expected Data record");
        }
    }
    #[test]
    fn test_record5() {
        let input = b"S5031234B6\n";
        let (input, rec) = parse_record5(input).unwrap();
        assert_eq!(input, b"");
        if let Record::Count(count) = rec {
            assert_eq!(count, 0x1234);
        } else {
            panic!("Expected Count record");
        }
    }
    #[test]
    fn test_record6() {
        let input = b"S6041234565F\n";
        let (input, rec) = parse_record6(input).unwrap();
        assert_eq!(input, b"");
        if let Record::Count(count) = rec {
            assert_eq!(count, 0x123456);
        } else {
            panic!("Expected Count record");
        }
    }
    #[test]
    fn test_record07() {
        let input = b"S70512001000D8";
        let (input, rec) = parse_record7(input).unwrap();
        assert_eq!(input, b"");
        match rec {
            Record::EOF(addr) => assert_eq!(addr, 0x12001000),
            _ => panic!("Expected EOF record"),
        }
    }
    #[test]
    fn test_record08() {
        let input = b"S804100000EB";
        let (input, rec) = parse_record8(input).unwrap();
        assert_eq!(input, b"");
        match rec {
            Record::EOF(addr) => assert_eq!(addr, 0x100000),
            _ => panic!("Expected EOF record"),
        }
    }
    #[test]
    fn test_record09() {
        let input = b"S9031234B6";
        let (input, rec) = parse_record9(input).unwrap();
        assert_eq!(input, b"");
        match rec {
            Record::EOF(addr) => assert_eq!(addr, 0x1234),
            _ => panic!("Expected EOF record"),
        }
    }
    #[test]
    fn test_s0_s1_s9_file() {
        let mut p = plugin();
        let mut file = p.open("srec://../../testing_binaries/rio/srec/record_0_1_9.srec", IoMode::READ).unwrap();
        assert_eq!(file.size, 0x142);
        let mut data = vec![0u8; 16];
        file.plugin_operations.read(0x1000, &mut data).unwrap();
        assert_eq!(data, [0x42, 0x79, 0x00, 0x00, 0x11, 0x42, 0x10, 0x3C, 0x00, 0x20, 0x12, 0x3C, 0x00, 0x00, 0x4E, 0x4F,]);
        file.plugin_operations.read(0x1070, &mut data).unwrap();
        assert_eq!(data, [0x67, 0xB0, 0x8A, 0xFC, 0x00, 0x3C, 0xBA, 0x7C, 0x00, 0x00, 0x66, 0x04, 0x3A, 0x3C, 0x00, 0x0C,]);
    }

    fn write_s0_s1_s9_cb(path: &Path) {
        let mut p = plugin();
        let uri = String::from("srec://") + &path.to_string_lossy();
        let mut file = p.open(&uri, IoMode::READ | IoMode::WRITE).unwrap();

        file.plugin_operations.write(0x1000, &[0x80, 0x90, 0xff, 0xfe]).unwrap();
        file.plugin_operations.write(0x1070, &[0x80, 0x90, 0xff, 0xfe]).unwrap();
        drop(file);
        file = p.open(&uri, IoMode::READ | IoMode::WRITE).unwrap();
        assert_eq!(file.size, 0x142);
        let mut data = vec![0u8; 16];
        file.plugin_operations.read(0x1000, &mut data).unwrap();
        assert_eq!(data, [0x80, 0x90, 0xff, 0xfe, 0x11, 0x42, 0x10, 0x3C, 0x00, 0x20, 0x12, 0x3C, 0x00, 0x00, 0x4E, 0x4F,]);
        file.plugin_operations.read(0x1070, &mut data).unwrap();
        assert_eq!(data, [0x80, 0x90, 0xff, 0xfe, 0x00, 0x3C, 0xBA, 0x7C, 0x00, 0x00, 0x66, 0x04, 0x3A, 0x3C, 0x00, 0x0C,]);
        file.plugin_operations.write(0x1000, &[0x42, 0x79, 0x00, 0x00]).unwrap();
        file.plugin_operations.write(0x1070, &[0x67, 0xB0, 0x8A, 0xFC]).unwrap();
        drop(file);
        let mut original = p.open("srec://../../testing_binaries/rio/srec/record_0_1_9.srec", IoMode::READ).unwrap();
        let mut file = p.open(&uri, IoMode::READ).unwrap();
        let mut d1 = [0; 0x142];
        let mut d2 = [0; 0x142];
        file.plugin_operations.read(0x1000, &mut d1).unwrap();
        original.plugin_operations.read(0x1000, &mut d2).unwrap();
        assert_eq!(d1[..], d2[..]);
    }

    #[test]
    fn test_write_s0_s1_s9() {
        operate_on_copy(&write_s0_s1_s9_cb, "../../testing_binaries/rio/srec/record_0_1_9.srec");
    }

    #[test]
    fn test_read_s2() {
        let mut p = plugin();
        let mut file = p.open("srec://../../testing_binaries/rio/srec/record_0_2_8.srec", IoMode::READ).unwrap();
        assert_eq!(file.size, 0x1f);
        let mut data = vec![0u8; 16];
        file.plugin_operations.read(0x111e4c, &mut data).unwrap();
        assert_eq!(data, [0xDF, 0x01, 0x08, 0x4E, 0x75, 0x48, 0xE7, 0x40, 0x00, 0x42, 0x41, 0xC0, 0xBC, 0x00, 0x00, 0x00]);
    }
    fn write_s2_cb(path: &Path) {
        let mut p = plugin();
        let uri = String::from("srec://") + &path.to_string_lossy();
        let mut file = p.open(&uri, IoMode::READ | IoMode::WRITE).unwrap();

        file.plugin_operations.write(0x111e4c, &[0x80, 0x90, 0xff, 0xfe]).unwrap();
        drop(file);
        file = p.open(&uri, IoMode::READ | IoMode::WRITE).unwrap();
        assert_eq!(file.size, 0x1f);
        let mut data = vec![0u8; 16];
        file.plugin_operations.read(0x111e4c, &mut data).unwrap();
        assert_eq!(data, [0x80, 0x90, 0xff, 0xfe, 0x75, 0x48, 0xE7, 0x40, 0x00, 0x42, 0x41, 0xC0, 0xBC, 0x00, 0x00, 0x00]);
        file.plugin_operations.write(0x111e4c, &[0xDF, 0x01, 0x08, 0x4E]).unwrap();
        drop(file);
        let mut original = p.open("srec://../../testing_binaries/rio/srec/record_0_2_8.srec", IoMode::READ).unwrap();
        let mut file = p.open(&uri, IoMode::READ).unwrap();
        let mut d1 = [0; 0x1f];
        let mut d2 = [0; 0x1f];
        file.plugin_operations.read(0x111e4c, &mut d1).unwrap();
        original.plugin_operations.read(0x111e4c, &mut d2).unwrap();
        assert_eq!(d1[..], d2[..]);
    }

    #[test]
    fn test_write_s2() {
        operate_on_copy(&write_s2_cb, "../../testing_binaries/rio/srec/record_0_2_8.srec");
    }

    #[test]
    fn test_read_s3() {
        let mut p = plugin();
        let mut file = p.open("srec://../../testing_binaries/rio/srec/record_0_3_7.srec", IoMode::READ).unwrap();
        assert_eq!(file.size, 0x1e);
        let mut data = vec![0u8; 16];
        file.plugin_operations.read(0x111e4cdf, &mut data).unwrap();
        assert_eq!(data, [0x01, 0x08, 0x4E, 0x75, 0x48, 0xE7, 0x40, 0x00, 0x42, 0x41, 0xC0, 0xBC, 0x00, 0x00, 0x00, 0xFF]);
    }

    fn write_s3_cb(path: &Path) {
        let mut p = plugin();
        let uri = String::from("srec://") + &path.to_string_lossy();
        let mut file = p.open(&uri, IoMode::READ | IoMode::WRITE).unwrap();

        file.plugin_operations.write(0x111e4cdf, &[0x80, 0x90, 0xff, 0xfe]).unwrap();
        drop(file);
        file = p.open(&uri, IoMode::READ | IoMode::WRITE).unwrap();
        assert_eq!(file.size, 0x1e);
        let mut data = vec![0u8; 16];
        file.plugin_operations.read(0x111e4cdf, &mut data).unwrap();
        assert_eq!(data, [0x80, 0x90, 0xff, 0xfe, 0x48, 0xE7, 0x40, 0x00, 0x42, 0x41, 0xC0, 0xBC, 0x00, 0x00, 0x00, 0xFF]);
        file.plugin_operations.write(0x111e4cdf, &[0x01, 0x08, 0x4E, 0x75]).unwrap();
        drop(file);
        let mut original = p.open("srec://../../testing_binaries/rio/srec/record_0_3_7.srec", IoMode::READ).unwrap();
        let mut file = p.open(&uri, IoMode::READ).unwrap();
        let mut d1 = [0; 0x1e];
        let mut d2 = [0; 0x1e];
        file.plugin_operations.read(0x111e4cdf, &mut d1).unwrap();
        original.plugin_operations.read(0x111e4cdf, &mut d2).unwrap();
        assert_eq!(d1[..], d2[..]);
    }

    #[test]
    fn test_write_s3() {
        operate_on_copy(&write_s3_cb, "../../testing_binaries/rio/srec/record_0_3_7.srec");
    }
}
