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

use hex::*;
#[derive(PartialEq,Clone)]
enum RSearchKeywordMode {
    Strings,
    Binary,
}
type TransformCB = fn(&RSearchKeyword, &[u8]) -> Vec<u8>;
/// RSearchKeyword struct stores keyword instance to search for in RSearch
pub struct RSearchKeyword {
    //TODO make this public restricted when it is avaialble
    mode: RSearchKeywordMode,
    #[doc(hidden)]
    pub bin_keyword: Vec<u8>,
    bin_mask: Vec<u8>,
    #[doc(hidden)]
    pub bad_char: Vec<i64>,
    #[doc(hidden)]
    pub transform: TransformCB,
}
impl RSearchKeyword {
    pub fn new_hex(kwhex: String, bitmask_hex: String) -> Result<RSearchKeyword, FromHexError> {
        let kwbuf = FromHex::from_hex(kwhex)?;
        let bitmaskbuf = FromHex::from_hex(bitmask_hex)?;
        let mut kw = RSearchKeyword::new(kwbuf, bitmaskbuf);
        kw.qs_badchar();
        Ok(kw)
    }
    pub fn new_str(kwbuf: String, bitmask_hex: String) -> Result<RSearchKeyword, FromHexError> {
        let bitmaskbuf: Vec<u8> = FromHex::from_hex(bitmask_hex)?;
        let mut kw = RSearchKeyword::new(kwbuf.as_bytes().to_vec(), bitmaskbuf);
        kw.mode = RSearchKeywordMode::Strings;
        kw.qs_badchar();
        return Ok(kw);
    }
    pub fn new_caseless_str(kwbuf: String) -> Result<RSearchKeyword, FromHexError> {
        let bitmaskbuf: Vec<u8> = Vec::new();
        let mut kw = RSearchKeyword::new(kwbuf.as_bytes().to_vec(), bitmaskbuf);
        kw.transform = RSearchKeyword::alpha_to_lower;
        kw.mode = RSearchKeywordMode::Strings;
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
        kw.qs_badchar();
        kw
    }
    fn qs_badchar(&mut self) {
        let buf = (self.transform)(self, &self.bin_keyword);
        self.bad_char.drain(..);
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
    pub fn is_string(&self) -> bool {
        self.mode == RSearchKeywordMode::Strings
    }
}

#[cfg(test)]
mod test {
    use super::*;
    lazy_static! {
        static ref KW1:RSearchKeyword = RSearchKeyword::new(vec![0x68, 0x65, 0x6c, 0x6c, 0x6f],vec![0x40, 0x40]);
        static ref KW2:RSearchKeyword = RSearchKeyword::new_hex("68656c6c6f".to_owned(), "4040".to_owned()).unwrap();
        static ref KW3:RSearchKeyword = RSearchKeyword::new_str("hello".to_owned(), "4040".to_owned()).unwrap();
        static ref KW4:RSearchKeyword = RSearchKeyword::new_caseless_str("H3LlO mY w0rLd".to_owned()).unwrap();
        static ref KW5:RSearchKeyword = RSearchKeyword::new_caseless_str("h3lLo My W0RlD".to_owned()).unwrap();
        static ref KW6:RSearchKeyword = RSearchKeyword::new_str("HeLlO".to_owned(), "".to_owned()).unwrap();
    }
    #[test]
    fn testing_new() {
        assert!(KW1.mode == RSearchKeywordMode::Binary);
        assert!(KW1.bin_keyword == vec![0x68, 0x65, 0x6c, 0x6c, 0x6f]);
        assert!(KW1.bin_mask == vec![0x40, 0x40]);
        assert!(KW1.bad_char.len() == 256);
    }
    #[test]
    fn testing_new_hex() {
        assert!(KW2.mode == RSearchKeywordMode::Binary);
        assert!(KW2.bin_keyword == vec![0x68, 0x65, 0x6c, 0x6c, 0x6f]);
        assert!(KW2.bin_mask == vec![0x40, 0x40]);
        assert!(!KW2.is_string());
        assert!(KW2.bad_char.len() == 256);
    }
    #[test]
    fn testing_buggy_new_hex() {
        let buggy_kw1 = RSearchKeyword::new_hex("68656c6c6".to_owned(), "4040".to_owned());
        assert!(buggy_kw1.is_err());
        let buggy_kw2 = RSearchKeyword::new_hex("68656c6c6f".to_owned(), "404".to_owned());
        assert!(buggy_kw2.is_err());
    }
    #[test]
    fn testing_new_str() {
        assert!(KW3.mode == RSearchKeywordMode::Strings);
        assert!(KW3.bin_keyword == vec![0x68, 0x65, 0x6c, 0x6c, 0x6f]);
        assert!(KW3.bin_mask == vec![0x40, 0x40]);
        assert!(KW3.is_string());
        assert!(KW3.bad_char.len() == 256);
    }
    #[test]
    fn testing_buggy_new_str() {
        let buggy_kw = RSearchKeyword::new_str("hello".to_owned(), "404".to_owned());
        assert!(buggy_kw.is_err());
    }
    #[test]
    fn testing_new_caseless_str() {
        assert!(KW4.mode == RSearchKeywordMode::Strings);
        assert!(KW4.bin_keyword ==
                vec![0x48, 0x33, 0x4c, 0x6c, 0x4f, 0x20, 0x6d, 0x59, 0x20, 0x77, 0x30, 0x72,
                     0x4c, 0x64]);
        assert!(KW4.bin_mask.is_empty());
        assert!(KW4.is_string());
        assert!(KW4.bad_char.len() == 256);
    }
    #[test]
    fn testing_len() {
        assert!(KW1.len() == 5);
        assert!(KW2.len() == 5);
        assert!(KW3.len() == 5);
        assert!(KW4.len() == 14);
        assert!(KW5.len() == 14);
    }
    #[test]
    fn testing_qs_badchar() {
        //TODO
    }
    #[test]
    fn testing_pass_as_it_is() {
        assert!((KW6.transform)(&KW6, &KW6.bin_keyword) == KW6.bin_keyword);
    }
    #[test]
    fn testing_alpha_to_lower() {
        assert!((KW4.transform)(&KW4, &KW4.bin_keyword) == (KW5.transform)(&KW5, &KW5.bin_keyword));
    }
    #[test]
    fn testing_do_mask() {
        assert!((KW1.transform)(&KW1, &KW1.bin_keyword) ==
                (KW1.transform)(&KW1, &[0x71, 0x72, 0x73, 0x74, 0x75]))
    }
}
