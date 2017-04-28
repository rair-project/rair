/*
 *  Copyright (C) 2017  Ahmed Abd El Mawgood
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use rustc_serialize::hex::*;
#[derive(PartialEq,Clone)]
enum RSearchKeywordMode {
    Strings,
    Binary,
}
type TransformCB = fn(&RSearchKeyword, &[u8]) -> Vec<u8>;
pub struct RSearchKeyword {
    mode: RSearchKeywordMode,
    pub bin_keyword: Vec<u8>, // same as bellow
    bin_mask: Vec<u8>,
    pub bad_char: Vec<i64>,
    pub transform: TransformCB,
}
impl RSearchKeyword {
    pub fn new_hex(kwstr: String, bitmask_str: String) -> Result<RSearchKeyword, FromHexError> {
        let kwbuf = kwstr.from_hex()?;
        let bitmaskbuf = bitmask_str.from_hex()?;
        let mut kw = RSearchKeyword::new(kwbuf, bitmaskbuf);
        kw.qs_badchar();
        Ok(kw)
    }
    pub fn new_str(kwbuf: String, bitmask_str: String) -> Result<RSearchKeyword, FromHexError> {
        let bitmaskbuf: Vec<u8> = bitmask_str.from_hex()?;
        let mut kw = RSearchKeyword::new(kwbuf.as_bytes().to_vec(), bitmaskbuf);
        kw.mode = RSearchKeywordMode::Strings;
        kw.qs_badchar();
        return Ok(kw);
    }
    pub fn new_caseless_str(kwbuf: String) -> Result<RSearchKeyword, FromHexError> {
        let bitmaskbuf: Vec<u8> = Vec::new();
        let mut kw = RSearchKeyword::new(kwbuf.as_bytes().to_vec(), bitmaskbuf);
        kw.transform = RSearchKeyword::alpha_to_lower;
        kw.qs_badchar();
        return Ok(kw);
    }
    pub fn len(&self) -> usize {
        return self.bin_keyword.len();
    }
    pub fn new(buf: Vec<u8>, mask: Vec<u8>) -> RSearchKeyword {
        let mut kw = RSearchKeyword {
            mode: RSearchKeywordMode::Binary,
            bin_keyword: buf,
             bin_mask: mask,
            bad_char: Vec::new(),
            transform: RSearchKeyword::pass_as_it_is,
        };
        if !kw.bin_mask.is_empty() {
            kw.transform = RSearchKeyword::do_mask;
        }
        kw
    }
    fn qs_badchar(&mut self) {
        let buf = (self.transform)(self, &self.bin_keyword);
        for _ in 0..256 {
            self.bad_char.push(buf.len() as i64 + 1);
        }
        for i in 0..buf.len() {
            self.bad_char[buf[i] as usize] = (buf.len() - i) as i64;
        }
    }
    fn pass_as_it_is(&self, buf: &[u8]) -> Vec<u8> {
        buf.to_vec()
    }
    fn alpha_to_lower(&self, buf: &[u8]) -> Vec<u8> {
        let mut new_buf: Vec<u8> = Vec::new();
        for byte in buf {
            if *byte >= 'A' as u8 && *byte <= 'Z' as u8 {
                new_buf.push(*byte | 32);
            } else {
                new_buf.push(*byte);
            }
        }
        new_buf
    }
    fn do_mask(&self, buf: &[u8]) -> Vec<u8> {
        let mut new_buf: Vec<u8> = Vec::new();
        for i in 0..buf.len() {
            new_buf.push(buf[i] & self.bin_mask[i % self.bin_mask.len()]);
        }
        new_buf
    }
}
