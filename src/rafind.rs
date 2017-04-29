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
extern crate getopts;
extern crate libc;
extern crate r_io;
extern crate r_util;
extern crate r_search;
extern crate r_cons;
extern crate rustc_serialize;
mod version;

use getopts::{Options, Matches};
use r_search::*;
use r_util::*;
use rustc_serialize::hex::ToHex;
use std::env;
use std::ptr;
#[derive(Clone)]
struct State {
    align: usize,
    format: OutputFormat,
    mask: String,
    bsize: usize,
    from: usize,
    to: usize,
    hexdump: bool,
    show_str: bool,
}
#[derive(PartialEq, Clone)]
enum EntryFormat {
    Hex,
    None,
}
struct SearchEntry {
    mode: r_search::SearchMode,
    format: EntryFormat,
    entry: String,
}
impl SearchEntry {
    fn new(mode: r_search::SearchMode, format: EntryFormat, entry: String) -> SearchEntry {
        SearchEntry {
            mode: mode,
            format: format,
            entry: entry,
        }
    }
}
fn argument_parser() -> Matches {
    let mut opts = Options::new();
    let matches;
    let args: Vec<String> = env::args().collect();
    opts.optopt("a", "", "only accept aligned hits", "align");
    opts.optopt("b", "", "set block size", "size");
    opts.optopt("e",
                "",
                "search for regular expression string matches",
                "regex");
    opts.optopt("f", "", "start searching from address 'from'", "from");
    opts.optflag("h", "", "show this help");
    opts.optflag("m", "", "magic search, file-type carver");
    opts.optopt("M",
                "",
                "set a binary mask to be applied on keywords",
                "str");
    opts.optflag("r", "", "print as radare2 flag commands");
    opts.optmulti("s",
                  "",
                  "search for a specific string (can be used multiple times)",
                  "str");
    opts.optopt("t", "", "stop search at address 'to'", "to");
    opts.optflag("v", "", "print version and exit");
    opts.optmulti("x",
                  "",
                  "search for hexpair string (can be used multiple times)",
                  "hex");
    opts.optflag("X", "", "show hexdump of search results");
    opts.optflag("z", "", "search for zero-terminated strings");
    opts.optflag("Z", "", "show string found on each search hit");
    match opts.parse(&args[1..]) {
        Ok(m) => matches = m,
        Err(f) => r_print::report(&f.to_string()),
    }
    if env::args().len() == 1 {
        let program = args[0].clone();
        let help = format!("Usage: {} [-mXnzZhv] [-a align] [-b sz] [-f/t from/\
                            to] [-[m|s|S|e] str] [-x hex] file ..",
                           program);
        r_print::report(&help);
    }
    if matches.opt_present("h") {
        let program = args[0].clone();
        let help = format!("Usage: {} [-mXnzZhv] [-a align] [-b sz] [-f/t from/\
                           to] [-[m|s|S|e] str] [-x hex] file ..",
                           program);
        r_print::report(&opts.usage(&help));
    }
    matches
}
fn get_search_entries(matches: &Matches) -> Vec<SearchEntry> {
    let mut search_list: Vec<SearchEntry> = Vec::new();
    if matches.opt_present("m") {
        search_list.push(SearchEntry::new(SearchMode::Magic, EntryFormat::None, String::new()));
    }
    if matches.opt_present("z") {
        search_list.push(SearchEntry::new(SearchMode::Strings, EntryFormat::None, String::new()));
    }
    if matches.opt_present("e") {
        let regexs = matches.opt_strs("e");
        for entry in regexs {
            search_list.push(SearchEntry::new(SearchMode::Regex, EntryFormat::None, entry));
        }
    }
    if matches.opt_present("s") {
        let strings = matches.opt_strs("s");
        for entry in strings {
            search_list.push(SearchEntry::new(SearchMode::Keyword, EntryFormat::None, entry));
        }
    }
    if matches.opt_present("x") {
        let hexstrings = matches.opt_strs("x");
        for entry in hexstrings {
            search_list.push(SearchEntry::new(SearchMode::Keyword, EntryFormat::Hex, entry));
        }
    }
    search_list
}
fn parse_state(matches: &Matches) -> State {
    let mut math = RNum::new(None, None, None);
    let mut state = State {
        align: 0,
        format: OutputFormat::None,
        mask: String::new(),
        bsize: 4096,
        from: 0,
        to: 0,
        hexdump: false,
        show_str: false,
    };
    if matches.opt_present("a") {
        let tmp = matches.opt_str("a").unwrap();
        state.align = match math.math(&tmp) {
            Ok(x) => x as usize,
            Err(y) => r_print::report(&y.to_string()),
        }
    }
    if matches.opt_present("r") {
        state.format = OutputFormat::Command;
    }
    if matches.opt_present("b") {
        let tmp = matches.opt_str("b").unwrap();
        state.bsize = match math.math(&tmp) {
            Ok(x) => x as usize,
            Err(y) => r_print::report(&y.to_string()),
        }
    }
    if matches.opt_present("M") {
        //There was little comment here saying
        //XXX should be from hexbin
        //Not sure what did it really meant
        state.mask = matches.opt_str("M").unwrap();
    }
    if matches.opt_present("f") {
        let tmp = matches.opt_str("f").unwrap();
        state.from = match math.math(&tmp) {
            Ok(x) => x as usize,
            Err(y) => r_print::report(&y.to_string()),
        }
    }
    if matches.opt_present("t") {
        let tmp = matches.opt_str("t").unwrap();
        state.to = match math.math(&tmp) {
            Ok(x) => x as usize + 1, //x didn't work so I guessed + 1 will
            Err(y) => r_print::report(&y.to_string()),
        }
    }
    if matches.opt_present("X") {
        state.hexdump = true;
    }
    if matches.opt_present("Z") {
        state.show_str = true;
    }
    state
}
fn hit(kw: &RSearchKeyword, addr: usize, buf: &[u8]) {
    let matches = argument_parser();
    let state = parse_state(&matches);
    if state.format == OutputFormat::Command {
        println!("f hit_{}_{:x} 0x{:x}", kw.bin_keyword.to_hex(), addr, addr);
        return;
    }

    if state.show_str {
        print!("Match at 0x{:08x}: ", addr);
        println!("{}", String::from_utf8_lossy(&kw.bin_keyword));
    } else {
        println!("Match at 0x{:08x}", addr);
    }
    if state.hexdump {
        let pr = r_print::new();
        let cur = addr % state.bsize;
        let end = if cur + 100 > buf.len() {
            buf.len()
        } else {
            cur + 100
        };
        r_print::hexdump(pr, addr, &buf[cur..end], 16, true);
    }

}
fn rafind_process(file: &str, list: &[SearchEntry], mut state: State) {
    let io = r_io::new();
    let mut rs: RSearch = RSearch::new();
    rs.set_callback(hit);
    if r_io::open_nomap(io, file, r_io::READ, 0) == ptr::null() {
        let errmsg = format!("Cannot open file '{}'", file);
        r_print::report(&errmsg);
    }
    r_cons::new(); //XXX there is nasty global variable manipulations here!
    rs.set_align(state.align);
    if state.to == 0 {
        state.to = r_io::size(io);
    }
    for entry in list {
        match entry.mode {
            SearchMode::Strings => {
                rs.regex_add(r"[[:print:]][[:print:]][[:print:]][[:print:]]*")
                    .unwrap();
            }
            SearchMode::Magic => {
                unimplemented!();
                //TODO lines 135-146
            }
            SearchMode::Keyword => insert_keyword(&mut rs, entry, &state),
            SearchMode::Regex => {
                match rs.regex_add(&entry.entry) {
                    Err(y) => r_print::report(&y.to_string()),
                    _ => (),
                }
            }
            _ => unimplemented!(),

        }
    }
    r_io::seek(io, state.from as u64, r_io::Seek::Set);
    let mut last = false;
    let mut cur = state.from;
    while !last && cur < state.to {
        if cur + state.bsize > state.to {
            state.bsize = state.to - cur;
            last = true;
        }
        rs.resize_buf(state.bsize); //TODO DELETE ME
        r_io::pread(io, cur as u64, rs.buf()); //XXX pread doesn't extend the buffer you know!;
        rs.search(cur);
        cur += state.bsize;
    }
}
fn insert_keyword(search: &mut RSearch, entry: &SearchEntry, state: &State) {
    let kw: RSearchKeyword;
    match entry.format {
        EntryFormat::Hex => {
            kw = match RSearchKeyword::new_hex(entry.entry.clone(), state.mask.clone()) {
                Ok(x) => x,
                Err(y) => r_print::report(&y.to_string()),
            }
        }
        EntryFormat::None => {
            kw = match RSearchKeyword::new_str(entry.entry.clone(), state.mask.clone()) {
                Ok(x) => x,
                Err(y) => r_print::report(&y.to_string()),
            }
        }
    }
    search.kw_add(kw).unwrap();
}

fn main() {
    let matches = argument_parser();
    let search_list = get_search_entries(&matches);
    let state = parse_state(&matches);
    if matches.opt_present("v") {
        let program = &env::args().nth(0).unwrap();
        version::blob_version(program);
        return;
    }
    for file in &matches.free {
        println!("Processing File {}:", file);
        rafind_process(file, &search_list, state.clone());
    }
}
