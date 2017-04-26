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
use std::io::{self, Write};
type RSearchCallback = fn(&RSearchKeyword, usize, &[u8]);
struct RKeywordHit {
    idx: usize,
    addr: usize,
}
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
    pub fn regex_add (&mut self, regex :&str) -> Result<usize, regex::Error>{
        self.regexs.push(Regex::new(regex)?);
        Ok(self.regexs.len())
    }
    pub fn begin(&mut self) {
        //SHOULD THIS EXIST ?
    }
    pub fn buf(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }
    pub fn resize_buf(&mut self, len: usize) {
        //GET THE FUCK RID OF THIS WHEN IO gets implemented
        if self.buf.len() != len {
            //because damn io should be vector aware
            self.buf.resize(len, 0);
        }
    }
    pub fn update(&mut self, from: usize) {
        //TODO one should hanle other types of searches here
        if !self.kws.is_empty() {
            self.quicksearch_update(from);
        }
        if !self.regexs.is_empty() {
            self.regexmatch_update(from);
        }
    }
    fn regexmatch_update(&mut self, from: usize) {
        let regex_len = self.regexs.len();
        for i in 0..regex_len {
            for mat in self.regexs[i].find_iter(&self.buf) {
                self.reghits.push(RegexHit{idx:i, start: from + mat.start(), end: from + mat.end()});
                let kw = RSearchKeyword::new(mat.as_bytes().to_vec(), Vec::new());
                match self.callback {
                    Some(cb) => cb(&kw, from + mat.start(), &self.buf),
                    None => (),
                }
            }
        }
    }
    fn quicksearch_update(&mut self, from: usize) {
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
            writeln!(&mut io::stderr(),
                     "Found new unaligned hit at 0x{:08x}",
                     addr)
                    .unwrap();
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
