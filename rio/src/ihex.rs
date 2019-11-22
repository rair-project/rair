/*
 * ihex.rs: RIO plugin that opens intel hex files.
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
use defaultplugin;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while_m_n;
use nom::combinator::map_res;
use nom::sequence::tuple;
use nom::IResult;
use plugin::*;
use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Write as IoWrite;
use std::path::Path;
use std::str;
use utils::*;
struct Dummy {}

impl RIOPluginOperations for Dummy {
    fn read(&mut self, _raddr: usize, _buffer: &mut [u8]) -> Result<(), IoError> {
        return Ok(());
    }
    fn write(&mut self, _raddr: usize, _buffer: &[u8]) -> Result<(), IoError> {
        return Ok(());
    }
}

const METADATA: RIOPluginMetadata = RIOPluginMetadata {
    name: "IHex",
    desc: "This IO plugin is used to open Intel IHex files,\
           this plugin would fill sparce intel ihex files with\
           zeros when doing read operation but in case of writes,\
           unfilled bytes will remain unfilled",
    author: "Oddcoder",
    license: "LGPL",
    version: "0.0.1",
};

struct FileInternals {
    file: Box<dyn RIOPluginOperations>, // defaultplugin
    uri: String,
    bytes: BTreeMap<u64, u8>, // sparce array of bytes
    prot: IoMode,
    ssa: Option<u32>, // used for Record 03
    sla: Option<u32>, // used for Record 05
}
named!(parse_newline, alt!(tag!("\r\n") | tag!("\n") | tag!("\r")));

enum Record {
    Data(u64, Vec<u8>), // Record 00 (base address, bytes)
    EOF,                // Record 01
    ESA(u64),           // Record 02
    SSA(u32),           //Record 03
    ELA(u64),           // Record 04
    SLA(u32),           // record 05
}
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
fn hex_big_dword(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, (word1, word2)) = tuple((hex_big_word, hex_big_word))(input)?;
    let result = ((word1 as u32) << 16) + word2 as u32;
    return Ok((input, result));
}
fn parse_record00(input: &[u8]) -> IResult<&[u8], Record> {
    // Data record
    let (input, _) = tag(":")(input)?;
    let (input, size) = hex_byte(input)?;
    let (input, addr) = hex_big_word(input)?;
    let (mut input, _) = tag("00")(input)?;
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

fn parse_record01(input: &[u8]) -> IResult<&[u8], Record> {
    // EOF Record
    let (input, _) = tag(":00")(input)?; // size entry
    let (input, _) = hex_big_word(input)?; // addr entry
    let (input, _) = tag("01")(input)?; // record ID
    let (input, _) = hex_byte(input)?; // checksum
    let (input, _) = parse_newline(input)?; //newline
    return Ok((input, Record::EOF));
}
fn parse_record02(input: &[u8]) -> IResult<&[u8], Record> {
    // Extended Segment Address Record
    let (input, _) = tag(":02")(input)?; // size entry
    let (input, _) = hex_big_word(input)?; // addr entry
    let (input, _) = tag("02")(input)?; // record ID
    let (input, addr) = hex_big_word(input)?; // data
    let (input, _) = hex_byte(input)?; // checksum
    let (input, _) = parse_newline(input)?; //newline
    return Ok((input, Record::ESA((addr as u64) << 4)));
}

fn parse_record03(input: &[u8]) -> IResult<&[u8], Record> {
    // Start Segment Address Record
    let (input, _) = tag(":04")(input)?; // size entry
    let (input, _) = hex_big_word(input)?; // addr entry
    let (input, _) = tag("03")(input)?; // record ID
    let (input, addr) = hex_big_dword(input)?; // data
    let (input, _) = hex_byte(input)?; // checksum
    let (input, _) = parse_newline(input)?; //newline
    return Ok((input, Record::SSA(addr)));
}

fn parse_record04(input: &[u8]) -> IResult<&[u8], Record> {
    // Extended Segment Address Record
    let (input, _) = tag(":02")(input)?; // size entry
    let (input, _) = hex_big_word(input)?; // addr entry
    let (input, _) = tag("04")(input)?; // record ID
    let (input, addr) = hex_big_word(input)?; // data
    let (input, _) = hex_byte(input)?; // checksum
    let (input, _) = parse_newline(input)?; //newline
    return Ok((input, Record::ELA((addr as u64) << 16)));
}

fn parse_record05(input: &[u8]) -> IResult<&[u8], Record> {
    // Start Linear Address Record
    let (input, _) = tag(":04")(input)?; // size entry
    let (input, _) = hex_big_word(input)?; // addr entry
    let (input, _) = tag("05")(input)?; // record ID
    let (input, addr) = hex_big_dword(input)?; // data
    let (input, _) = hex_byte(input)?; // checksum
    let (input, _) = parse_newline(input)?; //newline
    return Ok((input, Record::SLA(addr)));
}

impl FileInternals {
    fn parse_ihex(&mut self, input: &[u8]) -> Result<(), IoError> {
        named!(parse_record(&[u8]) -> Record, alt!(
            parse_record00 |
            parse_record01 |
            parse_record02 |
            parse_record03 |
            parse_record04 |
            parse_record05));
        let mut input = input;
        let mut base = 0u64;
        let mut line = 1;
        loop {
            let x = match parse_record(input) {
                Ok(x) => x,
                Err(_) => return Err(IoError::Custom(format!("Invalid Ihex entry at line: {}", line))),
            };
            input = x.0;
            match x.1 {
                Record::EOF => break,
                Record::Data(addr, data) => {
                    for i in 0..data.len() as u64 {
                        self.bytes.insert(i + addr + base, data[i as usize]);
                    }
                }
                Record::ESA(addr) => base = addr,
                Record::SSA(addr) => self.ssa = Some(addr),
                Record::ELA(addr) => base = addr,
                Record::SLA(addr) => self.sla = Some(addr),
            }
            line += 1;
        }
        return Ok(());
    }
    fn write_sa(&self, file: &mut File) -> Result<(), IoError> {
        if let Some(ssa) = self.ssa {
            let mut checksum: u16 = 4 + 3;
            for byte in ssa.to_be_bytes().iter() {
                checksum = (checksum + *byte as u16) & 0xFF;
            }
            checksum = 256 - checksum;
            writeln!(file, ":04000003{:08x}{:02x}", ssa, checksum)?;
        }
        if let Some(sla) = self.sla {
            let mut checksum: u16 = 4 + 5;
            for byte in sla.to_be_bytes().iter() {
                checksum = (checksum + *byte as u16) & 0xFF;
            }
            checksum = 256 - checksum;
            writeln!(file, ":04000005{:08x}{:02x}", sla, checksum)?;
        }
        return Ok(());
    }
    fn write_record04(&self, file: &mut File, addr: u64) -> Result<(), IoError> {
        let addr = (addr >> 16) as u16;
        let mut checksum = 6;
        for byte in addr.to_be_bytes().iter() {
            checksum = (checksum + *byte as u16) & 0xFF;
        }
        checksum = 256 - checksum;
        writeln!(file, ":02000004{:04x}{:02x}", addr, checksum)?;
        return Ok(());
    }
    fn write_record02(&self, file: &mut File, addr: u64) -> Result<(), IoError> {
        let addr = (addr >> 4) as u16;
        let mut checksum = 4;
        for byte in addr.to_be_bytes().iter() {
            checksum = (checksum + *byte as u16) & 0xFF;
        }
        checksum = 256 - checksum;
        writeln!(file, ":02000002{:04x}{:02x}", addr, checksum)?;
        return Ok(());
    }
    fn write_data(&self, file: &mut File) -> Result<(), IoError> {
        let mut checksum: u16 = 0x10;
        let mut addr = self.base();
        let mut data = String::new();
        let mut i = 0;
        for (k, v) in self.bytes.iter() {
            if i != 0 {
                if i == 0x10 || *k != addr + 1 {
                    writeln!(file, ":{:02x}{}{:02x}", i, data, 256 - checksum)?;
                    data.clear();
                    checksum = 0x10;
                    i = 0;
                } else {
                    // *k == addr + 1
                    write!(data, "{:02x}", *v).unwrap();
                    checksum = (checksum + *v as u16) & 0xff;
                }
            }
            if i == 0 {
                if *k > 0xfffff {
                    // record 04
                    self.write_record04(file, *k)?;
                } else if *k > 0xffff {
                    // record 02
                    self.write_record02(file, *k)?;
                }
                addr = *k;
                write!(data, "{:04x}{:02x}", *k & 0xffff, *v).unwrap();
                checksum = (checksum + *v as u16) & 0xff;
            }
            i += 1;
        }
        return Ok(());
    }
    fn save_ihex(&self) -> Result<(), IoError> {
        // truncate the current file.
        let mut file = OpenOptions::new().write(true).truncate(true).open(IHexPlugin::uri_to_path(&self.uri))?;
        //write ssa and sla
        self.write_sa(&mut file)?;
        //write data
        self.write_data(&mut file)?;
        // write EOF
        writeln!(file, ":00000001FF")?;
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
        return max - min;
    }
    fn base(&self) -> u64 {
        if let Some((k, _)) = self.bytes.iter().next() {
            return *k;
        } else {
            return 0;
        };
    }
}

impl RIOPluginOperations for FileInternals {
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
        // if we are dealing with cow or write firs write data to the sparce array
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
            self.save_ihex()?;
            // mmap new file
            let mut plug = defaultplugin::plugin();
            let def_desc = plug.open(&IHexPlugin::uri_to_path(&self.uri).to_string_lossy(), IoMode::READ)?;
            self.file = def_desc.plugin_operations;
        }
        return Ok(());
    }
}

struct IHexPlugin {
    defaultplugin: Box<dyn RIOPlugin>, // defaultplugin
}

impl IHexPlugin {
    fn uri_to_path(uri: &str) -> &Path {
        let path = uri.trim_start_matches("ihex://");
        return Path::new(path);
    }
    fn new() -> IHexPlugin {
        IHexPlugin {
            defaultplugin: defaultplugin::plugin(),
        }
    }
}

impl RIOPlugin for IHexPlugin {
    fn get_metadata(&self) -> &'static RIOPluginMetadata {
        return &METADATA;
    }

    fn open(&mut self, uri: &str, flags: IoMode) -> Result<RIOPluginDesc, IoError> {
        assert!(self.accept_uri(uri));
        let def_desc = self.defaultplugin.open(&IHexPlugin::uri_to_path(uri).to_string_lossy(), IoMode::READ)?;
        let mut internal = FileInternals {
            file: def_desc.plugin_operations,
            bytes: BTreeMap::new(),
            ssa: None,
            sla: None,
            prot: flags,
            uri: uri.to_string(),
        };
        let mut data = vec![0; def_desc.size as usize];
        internal.file.read(0x0, &mut data)?;
        internal.parse_ihex(&data)?;
        let desc = RIOPluginDesc {
            name: uri.to_owned(),
            perm: flags,
            raddr: internal.base(),
            size: internal.size(),
            plugin_operations: Box::new(internal),
        };
        return Ok(desc);
    }

    // either file:// or just no "://" to start with
    fn accept_uri(&self, uri: &str) -> bool {
        let split: Vec<&str> = uri.split("://").collect();
        if split.len() == 1 {
            return true;
        }
        if split[0] == "ihex" {
            return true;
        }
        return false;
    }
}

pub fn plugin() -> Box<dyn RIOPlugin> {
    return Box::new(IHexPlugin::new());
}
