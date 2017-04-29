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

use keyword::*;
use regex::bytes::*;
use regex;
type RSearchCallback = fn(&RSearchKeyword, usize, &[u8]);
struct RKeywordHit {
    idx: usize,
    addr: usize,
}
#[derive(Debug)]
struct RegexHit {
    idx: usize,
    start: usize,
    end: usize,
}
pub struct RSearch {
    align: usize,
    buf: Vec<u8>,
    callback: Option<RSearchCallback>,
    kws: Vec<RSearchKeyword>,
    kwhits: Vec<RKeywordHit>,
    regexs: Vec<Regex>,
    reghits: Vec<RegexHit>,
}

impl RSearch {
    pub fn new() -> RSearch {
        RSearch {
            align: 0,
            buf: Vec::new(),
            callback: None,
            kws: Vec::new(),
            kwhits: Vec::new(),
            regexs: Vec::new(),
            reghits: Vec::new(),
        }
    }
    pub fn set_buf(&mut self, buf: Vec<u8>) {
        self.buf = buf;
    }
    pub fn set_align(&mut self, align: usize) {
        self.align = align;
    }
    pub fn set_callback(&mut self, callback: RSearchCallback) {
        self.callback = Some(callback);
    }
    pub fn kw_add(&mut self, kw: RSearchKeyword) -> Result<usize, &'static str> {
        self.kws.push(kw);
        Ok(self.kws.len())
    }
    pub fn regex_add(&mut self, regex: &str) -> Result<usize, regex::Error> {
        self.regexs.push(Regex::new(regex)?);
        Ok(self.regexs.len())
    }
    /// Depricated dont use this, use `set_buf` instead.
    pub fn buf(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }
    /// Depricated dont use this, use `set_buf` instead.
    pub fn resize_buf(&mut self, len: usize) {
        //GET THE FUCK RID OF THIS WHEN IO gets implemented
        if self.buf.len() != len {
            //because damn io should be vector aware
            self.buf.resize(len, 0);
        }
    }
    pub fn search(&mut self, from: usize) {
        //TODO one should hanle other types of searches here
        self.kwhits.drain(..);
        self.reghits.drain(..);
        if !self.kws.is_empty() {
            self.quicksearch(from);
        }
        if !self.regexs.is_empty() {
            self.regexmatch(from);
        }
    }
    fn regexmatch(&mut self, from: usize) {
        let regex_len = self.regexs.len();
        for i in 0..regex_len {
            for mat in self.regexs[i].find_iter(&self.buf) {
                if self.align != 0 && (from + mat.start()) % self.align != 0 {
                    continue;
                }
                match self.callback {
                    Some(cb) => {
                        let kw = RSearchKeyword::new(mat.as_bytes().to_vec(), Vec::new());
                        cb(&kw, from + mat.start(), &self.buf);
                    }
                    None => (),
                }
                let hit = RegexHit {
                    idx: i,
                    start: from + mat.start(),
                    end: from + mat.end(),
                };
                self.reghits.push(hit);
            }
        }
    }
    fn quicksearch(&mut self, from: usize) {
        let len = self.buf.len();
        let kws_len = self.kws.len();
        for i in 0..kws_len {
            let mut offset = 0;
            let mut match_pos = 0;
            while offset < len && qs_find(&self.kws[i], offset, &self.buf, &mut match_pos) {
                self.add_kw_hit(i, from + match_pos);
                offset = match_pos + self.kws[i].len();
            }
        }
    }
    fn add_kw_hit(&mut self, idx: usize, addr: usize) {
        if self.align != 0 && addr % self.align != 0 {
            return;
        }
        match self.callback {
            Some(cb) => cb(&self.kws[idx], addr, &self.buf),
            None => (),
        }
        let hit = RKeywordHit {
            idx: idx,
            addr: addr,
        };
        self.kwhits.push(hit);
    }
}
fn qs_find(kw: &RSearchKeyword, offset: usize, buf: &[u8], match_pos: &mut usize) -> bool {
    let mut j = offset;
    let kw_buf = (kw.transform)(kw, &kw.bin_keyword);
    while j < buf.len() - kw.len() {
        let cmp_buf = (kw.transform)(kw, &buf[j..j + kw.len()]);
        if kw_buf == cmp_buf {
            *match_pos = j;
            return true;
        }
        j += kw.bad_char[(kw.transform)(kw, &[buf[j + kw.len()]])[0] as usize] as usize;
    }
    return false;
}
#[cfg(test)]
mod test {
    use super::*;
    lazy_static! {
       static ref BUF: Vec<u8> = vec![ //completely random!
0x8e, 0xe7, 0x3a, 0x08, 0x96, 0xb6, 0x42, 0x7d, 0xcc, 0x63, 0x39, 0x5c, 0x11, 0xd3, 0x9d, 0xf0,
0x19, 0xc7, 0x95, 0xe5, 0x31, 0xf5, 0x9c, 0x68, 0x99, 0xd4, 0x24, 0x77, 0x3c, 0xe4, 0x4f, 0x87,
0x00, 0xcc, 0x45, 0xa0, 0x0e, 0x18, 0xf4, 0x5a, 0xfe, 0x1a, 0x83, 0x93, 0x84, 0x45, 0xc7, 0x13,
0xb3, 0x5e, 0x90, 0x60, 0x63, 0xe9, 0x62, 0x46, 0xc3, 0xdb, 0x4c, 0x46, 0x30, 0xc2, 0x81, 0xe3,
0xa5, 0xd9, 0x67, 0x5c, 0xe1, 0x37, 0xbf, 0x5f, 0x69, 0x2d, 0x6b, 0xf5, 0x6d, 0xee, 0xe9, 0x4e,
0x91, 0xf8, 0x15, 0xb9, 0xb2, 0xc0, 0x19, 0xf8, 0xf0, 0x63, 0x77, 0xfb, 0x06, 0xa5, 0x41, 0x86,
0xad, 0x0e, 0xff, 0x6b, 0x9d, 0x3b, 0x02, 0x9e, 0xeb, 0xc0, 0x4c, 0xfc, 0x13, 0x1e, 0x35, 0xf7,
0xc2, 0xa4, 0xf4, 0xc3, 0xf2, 0x48, 0xb9, 0xc7, 0x71, 0x17, 0x92, 0x5b, 0xfa, 0x9c, 0x0b, 0xfe,
0xbc, 0x97, 0x8f, 0xa6, 0xa1, 0x24, 0x43, 0x53, 0xc5, 0x7e, 0xd7, 0x8a, 0x4b, 0xbc, 0x58, 0x52,
0xba, 0xe6, 0x9e, 0x05, 0x53, 0xfc, 0xc2, 0x24, 0xbd, 0x89, 0x28, 0x67, 0x85, 0x7a, 0xec, 0x06,
0xea, 0xc6, 0x13, 0x0b, 0xff, 0xaa, 0xbe, 0x15, 0x65, 0x8c, 0xa9, 0xa5, 0xc3, 0x96, 0x27, 0x0b,
0x82, 0xe3, 0x18, 0x80, 0xa8, 0x61, 0xd2, 0xd7, 0x07, 0x38, 0x40, 0x25, 0x2a, 0x31, 0xa6, 0xfb,
0x5f, 0x66, 0x5f, 0xa3, 0xfc, 0xa3, 0xb8, 0xcb, 0x5f, 0xaa, 0x36, 0x80, 0x4e, 0x5f, 0x0b, 0x6b,
0x19, 0xf4, 0x07, 0x9b, 0x08, 0x4e, 0x56, 0xb9, 0x0e, 0x2c, 0x01, 0xae, 0x32, 0x68, 0xc2, 0x94,
0x7d, 0x8e, 0x4b, 0xed, 0x75, 0xd2, 0x15, 0x38, 0x7b, 0x0c, 0xec, 0x36, 0xee, 0x67, 0x34, 0x24,
0x63, 0x4c, 0x3b, 0xe3, 0xe7, 0x2b, 0x75, 0xfc, 0x2e, 0x55, 0x69, 0x40, 0xf8, 0x4c, 0x9e, 0x76,
0xcd, 0x24, 0x95, 0x61, 0xda, 0x0d, 0x95, 0x20, 0x69, 0x4c, 0x97, 0xfc, 0x18, 0x02, 0x5d, 0xaa,
0x0e, 0xfd, 0x25, 0x2c, 0x7d, 0xb8, 0xac, 0x5d, 0x2c, 0x67, 0x47, 0x71, 0xb1, 0x1a, 0xa8, 0x76,
0xd7, 0x5a, 0x8b, 0xcb, 0xf0, 0x35, 0x59, 0x24, 0xfe, 0xdc, 0xc2, 0xb8, 0x51, 0xe5, 0xf2, 0x3a,
0xd1, 0xbf, 0x91, 0xf7, 0x13, 0xb7, 0x65, 0x07, 0xef, 0xae, 0x61, 0x58, 0xf0, 0x0c, 0x04, 0x3f,
0xdf, 0x8c, 0x63, 0xd5, 0x1a, 0x24, 0xab, 0xf5, 0x8e, 0xbf, 0xf6, 0xf9, 0xfa, 0xa8, 0x69, 0x28,
0x1d, 0x92, 0x2b, 0xec, 0x8a, 0x6a, 0xa3, 0x05, 0x52, 0xce, 0x17, 0xac, 0xda, 0x32, 0x6e, 0xe0,
0x25, 0x9b, 0xeb, 0x10, 0x65, 0x81, 0x26, 0x32, 0xc8, 0xd8, 0xe4, 0x94, 0x65, 0x9f, 0x5a, 0x29,
0x00, 0x2d, 0x18, 0xad, 0x3a, 0xc6, 0xee, 0x95, 0x4a, 0xfe, 0x85, 0xf1, 0x74, 0x8d, 0xc9, 0x13,
0x9e, 0xe2, 0x6c, 0x64, 0x62, 0xc7, 0x2b, 0x24, 0x2e, 0x90, 0x13, 0xfc, 0x28, 0xff, 0xfd, 0xc3,
0x26, 0x96, 0x02, 0x2f, 0x7a, 0x4d, 0x0e, 0x36, 0xa4, 0x87, 0x99, 0xc1, 0x01, 0xbc, 0xe8, 0x25,
0x01, 0xee, 0xce, 0x8e, 0xd5, 0x33, 0x0e, 0x28, 0x2d, 0xa8, 0x6f, 0x6b, 0x0b, 0x42, 0xbb, 0x08,
0xa4, 0xb3, 0x3d, 0x7c, 0xea, 0xd7, 0x0b, 0x24, 0x64, 0x37, 0xdd, 0x12, 0x9a, 0xaa, 0x8b, 0x14,
0x8b, 0xc3, 0x4d, 0x59, 0xda, 0x43, 0xb7, 0xc6, 0xb6, 0x03, 0x1c, 0x98, 0xec, 0xc7, 0x8a, 0x20,
0x27, 0xad, 0xe3, 0xad, 0x2c, 0x77, 0x8e, 0x58, 0x2e, 0x2a, 0x6c, 0xa9, 0xab, 0x29, 0xa4, 0x40,
0xfa, 0x6a, 0x30, 0xce, 0x2d, 0xa8, 0xa3, 0x98, 0xbc, 0x6b, 0xa8, 0x29, 0x9a, 0x1a, 0x23, 0x34,
0xc5, 0xc5, 0x7e, 0x87, 0x93, 0x27, 0xad, 0x43, 0x04, 0xbe, 0xa1, 0x1b, 0x1a, 0x09, 0xac, 0x5a,
0x0c, 0x3a, 0xc0, 0x08, 0x1f, 0x09, 0x36, 0x07, 0x2e, 0x2b, 0x73, 0x2d, 0x12, 0x77, 0x83, 0x85,
0xad, 0x77, 0x19, 0xa1, 0x8d, 0x5f, 0x6e, 0x33, 0x81, 0xdc, 0x7f, 0x22, 0xf1, 0xe1, 0xef, 0x21,
0xa5, 0xbb, 0x11, 0x47, 0x70, 0x01, 0xcf, 0xc0, 0x8b, 0x3e, 0x88, 0xac, 0xe8, 0x58, 0xcf, 0x38,
0xd1, 0x91, 0x6b, 0x10, 0x53, 0xfc, 0x0d, 0x10, 0xdc, 0x87, 0x9c, 0xeb, 0x9e, 0x2b, 0x58, 0x2b,
0x7e, 0x46, 0xe7, 0xa3, 0xef, 0x56, 0x64, 0xfd, 0x97, 0xaf, 0xbd, 0xfc, 0xdf, 0xd8, 0xba, 0x5d,
0x82, 0x8a, 0xf4, 0x6a, 0x82, 0xf3, 0x0e, 0xe4, 0x76, 0xec, 0xda, 0x69, 0xb9, 0xba, 0x9d, 0x53,
0x0b, 0x82, 0xfc, 0xaa, 0x9e, 0x47, 0x31, 0xc8, 0xf0, 0xb3, 0x16, 0xc4, 0xbf, 0x15, 0xa0, 0xd2,
0x1f, 0x73, 0x4c, 0xbd, 0x95, 0x04, 0x02, 0xee, 0xf9, 0xd9, 0x73, 0x62, 0xa0, 0xf7, 0x1c, 0x5e,
0x9c, 0x47, 0xc0, 0x98, 0x18, 0x2a, 0x6e, 0x1f, 0x34, 0x2a, 0x9e, 0x70, 0xb2, 0xe9, 0xb2, 0xa0,
0xc5, 0x9e, 0x11, 0x11, 0xa4, 0x48, 0xe7, 0x1f, 0x73, 0x30, 0x77, 0xa7, 0xff, 0xb0, 0x1a, 0x4b,
0x28, 0x47, 0x8b, 0x4d, 0x87, 0x4a, 0x07, 0xc2, 0x36, 0x68, 0x50, 0x6e, 0x89, 0xc1, 0x16, 0xcc,
0x19, 0x1d, 0x8d, 0xb3, 0xfb, 0x8a, 0xaf, 0xde, 0xd8, 0x51, 0xda, 0x12, 0x73, 0x7e, 0xf6, 0x2e,
0x8e, 0x0d, 0x56, 0x05, 0x0a, 0x80, 0x5c, 0xc7, 0xe9, 0xeb, 0x4f, 0x62, 0xe4, 0x5d, 0x27, 0x00,
0xd9, 0x59, 0xd8, 0x19, 0x6d, 0xdc, 0x28, 0x0f, 0xab, 0xad, 0xf2, 0xb8, 0x06, 0x67, 0x2d, 0x73,
0x79, 0x5c, 0x86, 0x96, 0x67, 0xe6, 0x39, 0xf9, 0xe1, 0xca, 0x3f, 0xd3, 0xfa, 0x7b, 0xe8, 0x64,
0x9c, 0xb2, 0x7a, 0x3c, 0xa8, 0x4e, 0x2e, 0x35, 0x6c, 0x8e, 0xa2, 0xa1, 0xb2, 0x62, 0xaf, 0x63,
0x8c, 0x86, 0x02, 0x55, 0x67, 0x4d, 0x42, 0x52, 0x27, 0x20, 0xe4, 0x9c, 0x5a, 0x10, 0x5f, 0x2b];
    }
    #[test]
    fn test_new() {
        let search = RSearch::new();
        assert!(search.align == 0);
        assert!(search.buf.is_empty());
        assert!(search.callback.is_none());
        assert!(search.kws.is_empty());
        assert!(search.kwhits.is_empty());
        assert!(search.regexs.is_empty());
        assert!(search.reghits.is_empty());
    }
    #[test]
    fn test_kw_add() {
        let mut search = RSearch::new();
        let kw1 = RSearchKeyword::new(vec![0x68, 0x65, 0x6c, 0x6c, 0x6f], vec![0x60]);
        let kw2 = RSearchKeyword::new(vec![0x6c, 0x64], vec![]);
        assert!(search.kw_add(kw1).unwrap() == 1);
        assert!(search.kw_add(kw2).unwrap() == 2);
        search.set_buf(BUF.to_vec());
        search.search(0);
        assert!(search.kwhits.len() == 3)
    }
    #[test]
    fn testing_regex_add() {
        let mut search = RSearch::new();
        search.set_buf(BUF.to_vec());
        search
            .regex_add(r"[[:print:]][[:print:]][[:print:]][[:print:]]*")
            .unwrap();
        search.search(0);
        assert!(search.reghits.len() == 27);
        search.set_align(4);
        search.search(0);
        assert!(search.reghits.len() == 5);
    }
}
