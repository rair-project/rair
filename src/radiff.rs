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
extern crate r_core;
extern crate r_util;

mod version;

use getopts::{Options, Matches};
use r_core::*;
use r_util::*;
use std::env;

#[derive(PartialEq)]
enum DiffMode {
    Code,
    Cols,
    Diff,
    Dist,
    Graph,
    Imports,
    Levenshtein,
    PDC,
}

#[derive(PartialEq)]
enum Verbosity {
    Quiet,
    Verbose,
    None,
}

//#[derive(Clone)]
struct State {
    arch:String,
    anal:bool,
    bits:u64,
    useva:bool,
    format:OutputFormat,
    show_count:bool,
    show_bare: bool,
    diffop:bool,
    disasm:bool,
    verbose: Verbosity
}

fn opencore(file: &String) ->RCore{
    let mut c = RCore::new();
    c.load_libs(LibScope::All, "".to_owned());
    //TODO
    c 
}

fn argument_parser() -> Matches {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    let matches;
    opts.optopt("a", "", "specify architecture plugin to use (x86, arm, ..)", "arch");
    opts.optflag("A", "", "run aaa after loading each binary");
    opts.optopt("b", "", "specify register size for arch (16, 32, 64, ..)", "bits");
    opts.optflag("c", "", "count of changes");
    opts.optflag("C", "", "graphdiff code (columns: off-A, match-ratio, off-B)");
    opts.optflag("d", "", "use delta diffing");
    opts.optflag("D", "", "show disasm instead of hexpairs");
    opts.optmulti("e", "", "set eval config var value for all RCore instances", "k=v");
    opts.optopt("g", "", "graph diff of given symbol, or between two offsets", "sym|off1,off");
    opts.optflag("h", "", "show this help");
    opts.optflag("i", "", "diff imports of target files (see -u, -U and -z)");
    opts.optflag("j", "", "output in json format");
    opts.optflag("l", "", "compute text distance (using levenshtein algorithm)");
    opts.optflag("n", "", "print bare addresses only (diff.bare=1)");
    opts.optflag("O", "", "code diffing with opcode bytes only");
    opts.optflag("p", "", "use physical addressing (io.va=0)");
    opts.optflag("P", "", "TODO document me"); //TODO
    opts.optflag("q", "", "quiet mode (disable colors, reduce output)");
    opts.optflag("r", "", "output in radare commands");    
    opts.optflag("s", "", "compute text distance");
    opts.optopt("S", "", "sort code diff (name, namelen, addr, size, type, dist) (only for -C or -g)", "name");
    opts.optopt("t", "", "set threshold for code diff (default is 70%)", "[0-100]");
    opts.optflag("u", "", "unified output (---+++)");
    opts.optflag("U", "", "unified output using system 'diff'");
    opts.optflag("v", "", "show version information");
    opts.optflag("V", "", "be verbose");
    opts.optflag("x", "", "show two column hexdump diffing");
    opts.optflag("z", "", "diff on extracted strings");
    match opts.parse(&args[1..]) {
        Ok(m) => matches = m,
        Err(f) => r_print::report(&f.to_string()),
    }
    if args.len() == 1 {
        let program = args[0].clone();
        let help = format!("Usage: {} [-abcCdjrspOxuUvV] [-A[A]] [-g sym] [-t %] [file] [file]", program);
        r_print::report(&help);
    }
    if matches.opt_present("h") {
        let program = args[0].clone();
        let help = format!("Usage: {} [-abcCdjlrspOxuUvV] [-A[A]] [-g sym] [-t %] [file] [file]", program);
        r_print::report(&opts.usage(&help));
    }
    matches
}

fn parse_state(matches: &Matches) -> State {
    let mut state = State {
        arch:String::new(),
        anal:false,
        bits:64,
        useva: true,
        format: OutputFormat::None,
        show_count: false,
        show_bare:false,
        diffop: false,
        disasm:false,
        verbose:Verbosity::None,
    };
    if matches.opt_present("a") {
        state.arch = matches.opt_str("i").unwrap();
    }
    if matches.opt_present("A") {
        state.anal = true;
    }
    if matches.opt_present("b") {
        state.bits = match matches.opt_str("b").unwrap().parse() {
            Ok(x) => x,
            Err(y) => r_print::report(&y.to_string()),
        };
    }
    if matches.opt_present("p") {
        state.useva = false;
    }
    if matches.opt_present("r") {
        state.format = OutputFormat::Command;
    }
    if matches.opt_present("c") {
        state.show_count = true;
    }
    if matches.opt_present("n") {
        state.show_bare = true;
    }
    if matches.opt_present("O") {
        state.diffop = true;
    }
    if matches.opt_present("D") {
        state.disasm = true;
    }
    if matches.opt_present("u") {
        state.format = OutputFormat::UniOut;
    }
    if matches.opt_present("q") {
        state.verbose = Verbosity::Quiet;
    }
    if matches.opt_present("V") {
        state.verbose = Verbosity::Verbose;
    }
    if matches.opt_present("j") {
        state.format = OutputFormat::Json;
    }
    let files: Vec<String> = matches.free.clone();
    if files.len() != 2 {
        r_print::report("Expected 2 file to diff against each other");
    }
    //TODO get started line 607
    state
}

fn handle_incompatiablity(matches: &Matches) {
    if matches.opt_present("g") && matches.opt_present("C") {
        r_print::report("`-g` and `-C` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("i") && matches.opt_present("C") {
        r_print::report("`-i` and `-C` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("g") && matches.opt_present("i") {
        r_print::report("`-g` and `-i` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("P") && matches.opt_present("C") {
        r_print::report("`-P` and `-C` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("P") && matches.opt_present("g") {
        r_print::report("`-P` and `-g` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("P") && matches.opt_present("i") {
        r_print::report("`-P` and `-i` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("g") && matches.opt_present("s") {
        r_print::report("`-g` and `-s` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("i") && matches.opt_present("s") {
        r_print::report("`-i` and `-s` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("s") && matches.opt_present("C") {
        r_print::report("`-s` and `-C` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("P") && matches.opt_present("s") {
        r_print::report("`-P` and `-s` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("g") && matches.opt_present("l") {
        r_print::report("`-g` and `-l` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("i") && matches.opt_present("l") {
        r_print::report("`-i` and `-l` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("P") && matches.opt_present("l") {
        r_print::report("`-P` and `-l` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("l") && matches.opt_present("s") {
        r_print::report("`-l` and `-s` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("l") && matches.opt_present("C") {
        r_print::report("`-l` and `-C` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("x") && matches.opt_present("C") {
        r_print::report("`-x` and `-C` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("i") && matches.opt_present("x") {
        r_print::report("`-i` and `-x` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("g") && matches.opt_present("x") {
        r_print::report("`-g` and `-x` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("P") && matches.opt_present("x") {
        r_print::report("`-P` and `-x` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("x") && matches.opt_present("s") {
        r_print::report("`-x` and `-s` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("x") && matches.opt_present("l") {
        r_print::report("`-x` and `-l` are not compatiable, you can not \
            use both of them at the same time");
    }

    if matches.opt_present("r") && matches.opt_present("j") {
        r_print::report("`-r` and `-j` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("r") && matches.opt_present("u") {
        r_print::report("`-r` and `-u` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("r") && matches.opt_present("U") {
        r_print::report("`-r` and `-U` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("j") && matches.opt_present("u") {
        r_print::report("`-j` and `-u` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("u") && matches.opt_present("U") {
        r_print::report("`-u` and `-U` are not compatiable, you can not \
            use both of them at the same time");
    }
    if matches.opt_present("V") && matches.opt_present("q") {
        r_print::report("`-V` and `-q` are not compatiable, you can not \
            use both of them at the same time");
    }
}

fn main() {
    let matches = argument_parser();
    handle_incompatiablity(&matches);
    let mut state = parse_state(&matches);
    let mut evals:Vec<String> = Vec::new();
    let mut mode = DiffMode::Diff;
    let mut addr = String::new();
    let mut threshold:i8 = 70;
    let mut delta = false;
    let mut columen_sort = String::new();
    if matches.opt_present("v") {
        let program = &env::args().nth(0).unwrap();
        version::blob_version(program);
    }
    if matches.opt_present("e") {
        evals.append(&mut matches.opt_strs("e"));
    }
    if matches.opt_present("g") {
        mode = DiffMode::Graph;
        addr = matches.opt_str("g").unwrap();
    }
    if matches.opt_present("C") {
        mode = DiffMode::Code;
    }
    if matches.opt_present("i") {
        mode = DiffMode::Imports;
    }
    if matches.opt_present("t") {
        threshold = match matches.opt_str("t").unwrap().parse() {
            Ok(x) => x,
            Err(y) => r_print::report(&y.to_string()),
        };
    }
    if matches.opt_present("d") {
        delta = true;
    }
    if matches.opt_present("P") {
        mode = DiffMode::PDC;
        //XXX real mode was CODE,
        //disasm = false
        //pdc = true
    }
   if matches.opt_present("s") {
       mode = DiffMode::Dist;
   }
    if matches.opt_present("l") {
        mode = DiffMode::Levenshtein;
    }
    if matches.opt_present("S") {
        columen_sort = matches.opt_str("S").unwrap();
    }
    if matches.opt_present("x") {
        mode = DiffMode::Cols;
    }
}
