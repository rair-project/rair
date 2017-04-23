/*
 *  rahash.rs -- Block based hashing utility
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
extern crate r_crypto;
extern crate r_hash;
extern crate r_io;
extern crate r_util;
//TODO get rid of rustc_serialize
extern crate rustc_serialize;

mod version;

use getopts::{Options, Matches};
use libc::*;
use r_hash::RHash;
use r_util::*;
use r_util::r_num::RNum;
use rustc_serialize::hex::FromHex;
use std::env;
use std::io::{self, Read, Write};
use std::fs::File;
use std::ptr;
#[derive(PartialEq,Clone)]
enum OutputFormat {
    Json,
    Command,
    Ssh,
    None,
}
#[derive(Clone)]
struct Status {
    quiet: bool,
    iterations: u64,
    format: OutputFormat,
    incremental: bool,
    little_endian: bool,
    from: usize,
    to: usize,
}

fn algolist() {
    //TODO ther should be better way than bits magic ..
    //I mean come on store it in a hashmap or something ...
    //with a well defined APIs
    println!("Available Hashes:");
    for i in 0..r_hash::R_HASH_NBITS {
        let bits: u64 = 1 << i;
        let name = r_hash::name(bits);
        if !name.is_empty() {
            println!("  {}", name);
        }
    }
    println!("Available Encoders/Decoders:");
    println!("  base64\n  base91\n  punycode\nAvailable Crypto Algos:");
    for i in 0..64 {
        let bits: u64 = 1 << i;
        let name = r_crypto::name(bits);
        if !name.is_empty() {
            println!("  {}", name);
        }
    }
}

fn do_hash_hexprint(c: &[u8], little_endian: bool) {
    if little_endian {
        //TODO BUGGY I believe this code might not do exactly what it promises
        //the reason is that it is not the same as what is in r2 source but I
        //believe it is logical so please if you have the same feeling as me
        //.. let me know!
        let mut i = 0;
        while i < c.len() {
            print!("{:02x}{:02x}{:02x}{:02x}",
                   c[i + 3],
                   c[i + 2],
                   c[i + 1],
                   c[i]);
            i += 4;
        }

    } else {
        for i in c {
            print!("{:02x}", i);
        }
    }
}
fn do_hash_print(ctx: &r_hash::RHash, algo: u64, len: usize, status: &Status) {
    let hname = r_hash::name(algo);
    let mut c: Vec<u8> = (*ctx).digest.to_vec();
    c.drain(len..);
    match status.format {
        OutputFormat::None => {
            if !status.quiet {
                print!("0x{:08x}-0x{:08x} {}: ", status.from, status.to - 1, hname);
            }
            do_hash_hexprint(&c, status.little_endian);
            println!("");
        }
        OutputFormat::Json => {
            print!("{{\"name\":\"{}\",\"hash\":\"", hname);
            do_hash_hexprint(&c, status.little_endian);
            println!("\"}}");
        }
        OutputFormat::Command => {
            print!("e file.{}=", hname);
            do_hash_hexprint(&c, status.little_endian);
            println!("");
        }
        OutputFormat::Ssh => {
            let art = r_print::randomart(&c, status.from);
            println!("{}\n{}", hname, art);
        }
    };
}

fn do_hash_internal(ctx: &r_hash::RHash, algo: u64, buf: &[u8], print: bool,status: &Status, s: &r_hash::RHashSeed) {
    let dlen = ctx.calculate(algo, buf);
    if dlen == 0 {
        return;
    }
    if !print {
        return;
    }
    if algo == r_hash::R_HASH_ENTROPY {
        let e = r_hash::entropy(buf);
        if status.format == OutputFormat::None {
            print!("0x{:08x}-0x{:08x} {:.10}", status.from, status.to - 1, e);
            r_print::progressbar(ptr::null(), (12.5 * e) as i32, 60);
            println!("");
        } else {
            //TODO that doesn't look either radare2 commands or json tbh
            println!("entropy: {:.10}", e);
        }
    } else {
        if status.iterations > 0 {
            ctx.do_spice(algo, status.iterations, s);
        }
        do_hash_print(ctx, algo, dlen, status);

    }
}

fn compare_hashes(ctx: &r_hash::RHash, compare: &[u8], len: usize) {
    let mut c: Vec<u8> = (*ctx).digest.to_vec();
    c.drain(len..);
    if c == compare {
        println!("Computed hash matches the expected one.");
    } else {
        println!("Computed hash doesn't match the expected one.");
    }
}

fn do_hash(file: &str,
           algo: &str,
           io: &c_void,
           mut bsize: usize,
           compare: &[u8],
           mut status: &mut Status,
           s: &r_hash::RHashSeed) {
    let algobit = r_hash::name_to_bits(algo);
    if algobit == r_hash::R_HASH_NONE {
        r_print::report("Invalid hashing algorithm ");
    }
    let fsize = r_io::size(io);
    if bsize == 0 || bsize > fsize {
        bsize = fsize;
    }
    if status.to == 0 {
        status.to = fsize;
    }
    if status.from > status.to {
        r_print::report("Invalid -f -t range");
    }
    let ctx = RHash::new(true, algobit);
    if status.format == OutputFormat::Json {
        print!("[");
    }
    if status.incremental {
        let mut i = 1;
        let mut first = true;
        while i < r_hash::R_HASH_ALL {
            let hashbit = algobit & i;
            if hashbit != 0 {
                let dlen = r_hash::size(algobit);
                ctx.do_begin(i);
                if status.format == OutputFormat::Json {
                    if first {
                        first = false;
                    } else {
                        print!(",");
                    }
                }
                if s.prefix & !s.buf.is_empty() {
                    do_hash_internal(ctx, hashbit, &s.buf, false, status, s);
                }
                let mut j = status.from;
                while j < status.to {
                    let nsize = if j + bsize > status.to {
                        status.to - j
                    } else {
                        bsize
                    };
                    let buf: Vec<u8> = vec![0; nsize];
                    r_io::pread(io, j as u64, &buf);
                    do_hash_internal(ctx, hashbit, &buf, false, status, s);
                    j += bsize;
                }
                if s.prefix & !s.buf.is_empty() {
                    do_hash_internal(ctx, hashbit, &s.buf, false, status, s);
                }
                ctx.do_end(i);
                if status.iterations > 0 {
                    ctx.do_spice(i, status.iterations, s);
                }
                if !status.quiet && status.format != OutputFormat::Json {
                    print!("{} ", file);
                }
                do_hash_print(ctx, i, dlen, status);
            }
            i <<= 1;
        }
    } else {
        if !s.buf.is_empty() {
            r_print::report("Seed cannot be used on per-block hashing.");
        }

        let mut i = 1;
        while i < r_hash::R_HASH_ALL {
            let hashbit = algobit & i;
            if hashbit != 0 {
                let mut j = status.from;
                let mut status_c = status.clone();
                while j < status.to {
                    let nsize = if j + bsize < fsize { bsize } else { fsize - j };
                    let buf: Vec<u8> = vec![0; nsize];
                    r_io::pread(io, j as u64, &buf);
                    status_c.from = j;
                    status_c.to = j + bsize;
                    if status_c.to > fsize {
                        status_c.to = fsize;
                    }
                    do_hash_internal(ctx, hashbit, &buf, true, &status_c, s);
                    j += bsize;
                }
            }
            i <<= 1;
        }
    }
    if status.format == OutputFormat::Json {
        println!("]");
    }
    if !compare.is_empty() {
        let hash_size = r_hash::size(algobit);
        compare_hashes(ctx, compare, hash_size);
    }

}

fn encrypt_or_decrypt_file(algo: &str,
                           is_decryption: bool,
                           file: &str,
                           iv: &[u8],
                           s: &r_hash::RHashSeed) {
    let mut buf: Vec<u8> = Vec::new();
    match file {
        "-" => {
            io::stdin().read(&mut buf).unwrap();
        }
        _ => {
            let mut file = match File::open(file) {
                Ok(f) => f,
                Err(why) => r_print::report(&why.to_string()),
            };
            file.read(&mut buf).unwrap();
        }
    }
    encrypt_or_decrypt(algo, is_decryption, &buf, iv, s);
}

fn encrypt_or_decrypt(algo: &str,
                      is_decryption: bool,
                      buf: &[u8],
                      iv: &[u8],
                      s: &r_hash::RHashSeed) {
    //TODO find better way ..
    if !(&*algo == "base64" || &*algo == "base91" || &*algo == "punycode") && s.buf.is_empty() {
        if is_decryption {
            r_print::report("Decryption key is not defined. Use -S [key]");
        } else {
            r_print::report("Encryption key is not defined. Use -S [key]");
        }
    }
    let cry = r_crypto::new();
    if !r_crypto::use_algo(cry, algo) {
        if is_decryption {
            let err = format!("Unknown decryption algorithm '{}'", algo);
            r_print::report(&*err);
        } else {
            let err = format!("Unknown encryption algorithm '{}'", algo);
            r_print::report(&*err);
        }
    }
    if !r_crypto::set_key(cry, &s.buf, 0, is_decryption) {
        r_print::report("Invalid key");
    }
    if !iv.is_empty() && !r_crypto::set_iv(cry, iv) {
        r_print::report("Invalid initialization vector");
    }
    r_crypto::update(cry, buf);
    r_crypto::finish(cry, &Vec::new());
    let result = r_crypto::get_output(cry);
    std::io::stdout().write(&result).unwrap();
}

fn do_hash_seed(mut seed: String) -> r_hash::RHashSeed {
    let mut r_hash_seed = r_hash::RHashSeed::new();
    if seed.is_empty() {
        return r_hash_seed;
    }
    if seed.starts_with('-') {
        io::stdin().read(&mut r_hash_seed.buf).unwrap();
        return r_hash_seed;
    }
    if seed.starts_with('^') {
        r_hash_seed.prefix = true;
        seed.remove(0);
    } else {
        r_hash_seed.prefix = false;
    }
    if seed.starts_with("S:") {
        seed.drain(0..2);
        r_hash_seed.buf.extend(seed.as_bytes());
    } else {
        r_hash_seed.buf = match (*seed).from_hex() {
            Ok(buf) => buf,
            Err(why) => r_print::report(&(why.to_string())),
        }
    }
    r_hash_seed
}

fn is_power_of_two(x: u64) -> bool {
    (x != 0) && (x & (x - 1)) == 0
}
fn argument_parser() -> Matches {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    let matches;
    opts.optopt("a",
                "",
                "comma separated list of algorithms (default is 'sha256')",
                "algo");
    opts.optflag("B", "", "show per-block hash");
    opts.optopt("b",
                "",
                "specify the size of the block (instead of full file)",
                "bsize");
    opts.optopt("c", "", "compare with this hash", "hash");
    opts.optopt("d",
                "",
                "decrypt. Use -S to set key and -I to set IV",
                "algo");
    opts.optflag("E", "", "use as little endian");
    opts.optopt("e",
                "",
                "encrypt. Use -S to set key and -I to set IV",
                "algo");
    opts.optopt("f", "", "start hashing at given address", "from");
    opts.optflag("h", "", "print this help message");
    opts.optopt("I",
                "",
                "use give initialization vector (IV) (hexa or s:string)",
                "iv");
    opts.optopt("i", "", "repeat hash N iterations", "num");
    opts.optflag("j", "", "output in JSON format");
    opts.optflag("k", "", "show hash using the openssh's randomkey algorithm");
    opts.optflag("l", "", "list all available algorithms (see -a)");
    opts.optflag("q", "", "run in quiet mode (-qq to show only the hash)");
    opts.optflag("r", "", "output radare commands");
    opts.optopt("S",
                "",
                "use given seed for hasing or key for encryption / \
        decryption (hexa or s:string) use ^ to use seed as prefix (key for -E) \
                (- will slurp the key from stdin.",
                "seed");
    opts.optopt("s", "", "hash this string instead of files", "string");
    opts.optopt("t", "", "stop hashing at given address", "to");
    opts.optflag("v", "", "show version information");
    opts.optopt("x",
                "",
                "hash this hexpair string instead of files",
                "hexpair");
    match opts.parse(&args[1..]) {
        Ok(m) => matches = m,
        Err(f) => r_print::report(&f.to_string()),
    };
    if matches.opt_present("h") {
        let program = args[0].clone();
        let help = format!("Usage: {} [-rBhlkvje] [-b S] [-a A] [-c H] [-e A] \
                           [-s S] [-f O] [-t O] [file] ...",
                           program);
        r_print::report(&opts.usage(&help));
    }
    matches
}
fn parse_status(matches: &Matches) -> Status {
    let mut math = RNum::new(None, None, None);
    let mut status: Status = Status {
        quiet: false,
        iterations: 0,
        format: OutputFormat::None,
        incremental: true,
        little_endian: false,
        from: 0,
        to: 0,
    };
    if matches.opt_present("q") {
        status.quiet = true;
    }
    if matches.opt_present("i") {
        let tmp = matches.opt_str("i").unwrap();
        match (&tmp).parse() {
            Ok(m) => status.iterations = m,
            Err(f) => r_print::report(&f.to_string()),
        }
    }
    if matches.opt_present("j") {
        match status.format {
            OutputFormat::None => status.format = OutputFormat::Json,
            _ => {
                r_print::report("`-j`, `-r` and `-k` are not compatiable, you can not \
                        use any two of them at the same time")
            }
        }
    }
    if matches.opt_present("r") {
        match status.format {
            OutputFormat::None => status.format = OutputFormat::Command,
            _ => {
                r_print::report("`-j`, `-r` and `-k` are not compatiable, you can not \
                        use any two of them at the same time")
            }
        }
    }
    if matches.opt_present("k") {
        match status.format {
            OutputFormat::None => status.format = OutputFormat::Ssh,
            _ => {
                r_print::report("`-j`, `-r` and `-k` are not compatiable, you can not \
                        use any two of them at the same time")
            }
        }
    }
    if matches.opt_present("B") {
        status.incremental = false;
    }
    if matches.opt_present("t") {
        let tmp = matches.opt_str("t").unwrap();
        status.to = match math.math(&tmp) {
            Ok(x) => x as usize + 1,
            Err(y) => r_print::report(&y.to_string()),
        };
    }
    if matches.opt_present("f") {
        let tmp = matches.opt_str("f").unwrap();
        status.from = match math.math(&tmp) {
            Ok(x) => x as usize,
            Err(y) => r_print::report(&y.to_string()),
        };
    }
    if status.to != 0 && status.from >= status.to {
        r_print::report("Invalid -f or -t offsets\n");
    }
    if matches.opt_present("E") {
        status.little_endian = true;
    }
    status
}
fn main() {
    //TODO option n that that I dont really know what is the high level description of its
    //behaviour
    let mut hashstr = String::new();
    let mut compare_str = String::new();
    let mut decrypt: String = String::new();
    let mut encrypt: String = String::new();
    let mut ivseed: String = String::new();
    let mut ishex = false;
    let mut bsize: usize = 0;
    let mut algo = "sha256".to_owned();
    let mut seed: String = String::new();
    let mut iv: Vec<u8> = Vec::new();
    let mut hash: Vec<u8> = Vec::new();
    let mut compare_bin: Vec<u8> = Vec::new();
    let matches = argument_parser();
    let mut status = parse_status(&matches);
    if matches.opt_present("l") {
        algolist();
        return;
    }
    if matches.opt_present("v") {
        let program = &env::args().nth(0).unwrap();
        version::blob_version(program);
        return;
    }
    if matches.opt_present("S") {
        seed = matches.opt_str("S").unwrap();
    }
    if matches.opt_present("I") {
        ivseed = matches.opt_str("I").unwrap();
    }
    if matches.opt_present("d") {
        decrypt = matches.opt_str("d").unwrap();
    }
    if matches.opt_present("e") {
        encrypt = matches.opt_str("e").unwrap();
    }
    if matches.opt_present("a") {
        algo = matches.opt_str("a").unwrap();
    }
    if matches.opt_present("b") {
        let mut math = RNum::new(None, None, None);
        let tmp = matches.opt_str("b").unwrap();
        bsize = match math.math(&tmp) {
            Ok(x) => x as usize,
            Err(y) => r_print::report(&y.to_string()),
        };
    }
    if matches.opt_present("s") && matches.opt_present("x") {
        r_print::report(" -s and -x are not compatiable, you can not \
        use both of them at the same time");
    }
    if matches.opt_present("s") {
        hashstr = matches.opt_str("s").unwrap();
        ishex = false;
    }
    if matches.opt_present("x") {
        hashstr = matches.opt_str("x").unwrap();
        ishex = true;
    }
    if matches.opt_present("c") && matches.opt_present("b") && matches.opt_present("B") {
        r_print::report("Option -c incompatible with -b and -B options.");
    }
    if matches.opt_present("c") {
        compare_str = matches.opt_str("c").unwrap();
    }
    if matches.opt_present("e") && matches.opt_present("d") {
        r_print::report("Option -e and -d are incompatible with each other.")
    }
    if !compare_str.is_empty() {
        let algobit: u64;
        if &encrypt == "base64" || &encrypt == "base91" || &decrypt == "base64" ||
           &decrypt == "base91" {
            r_print::report("Option -c incompatible with -E base64, -E base91, -D base64 or \
                   -D base91 options.");
        }
        algobit = r_hash::name_to_bits(&algo);
        if !is_power_of_two(algobit) {
            r_print::report("Option -c incompatible with multiple algorithms in -a.");
        }
        compare_bin = match (*compare_str).from_hex() {
            Err(why) => r_print::report(&why.to_string()),
            Ok(x) => x,
        };
        if compare_bin.len() != r_hash::size(algobit) {
            let err_msg = format!("rahash2: Given -c hash has {} bytes but the \
                selected algorithm returns {} bytes.",
                                  compare_bin.len(),
                                  r_hash::size(algobit));
            r_print::report(&err_msg);
        }
    }
    if !ivseed.is_empty() {
        if ivseed.starts_with("s:") {
            iv.extend(ivseed[2..].as_bytes());
        } else {
            iv = match (*ivseed).from_hex() {
                Err(why) => r_print::report(&why.to_string()),
                Ok(x) => x,
            }
        }
    }
    let hash_seed = do_hash_seed(seed);
    if !hashstr.is_empty() {
        if &hashstr == "-" {
            io::stdin().read_to_string(&mut hashstr).unwrap();
        }
        if ishex {
            hash = match (*hashstr).from_hex() {
                Err(why) => r_print::report(&why.to_string()),
                Ok(x) => x,
            }
        } else {
            hash.extend(hashstr.as_bytes());
        }
        if status.from >= hash.len() {
            r_print::report("-f value is greater than hash length");
        }
        if status.to > hash.len() {
            r_print::report("-t value is greater than hash length");
        }
        if status.to == 0 {
            status.to = hash.len();
        }
        hash.drain(0..status.from);
        hash.drain(status.to..);
        if !encrypt.is_empty() {
            encrypt_or_decrypt(&encrypt, false, &hash, &iv, &hash_seed);
            return;
        } else if !decrypt.is_empty() {
            encrypt_or_decrypt(&decrypt, false, &hash, &iv, &hash_seed);
            return;
        } else {
            let full_str: Vec<u8>;
            if !hash_seed.buf.is_empty() {
                if hash_seed.prefix {
                    full_str = [hash_seed.buf.clone(), hash].concat();
                } else {
                    full_str = [hash, hash_seed.buf.clone()].concat();
                }
            } else {
                full_str = hash;
            }
            let algobit = r_hash::name_to_bits(&algo);
            if algobit == 0 {
                r_print::report("Invalid algorithm. See -E and -D");
            }
            //TODO THIS SUCKS SHOULD BE IMPROVED FOR SURE
            let mut i = 1;
            while i < r_hash::R_HASH_ALL {
                let hashbit = algobit & i;
                if hashbit != 0 {
                    let ctx = RHash::new(true, hashbit);
                    status.from = 0;
                    status.to = full_str.len();
                    do_hash_internal(ctx,
                                     hashbit,
                                     &full_str,
                                     true,
                                     &status,
                                     &hash_seed);
                    if !compare_bin.is_empty() {
                        let hash_size = r_hash::size(algobit);
                        compare_hashes(ctx, &compare_bin, hash_size);
                    }
                }
                i <<= 1;
            }
            return;
        }
    }
    for file in &matches.free {
        let io = r_io::new();
        if !encrypt.is_empty() {
            encrypt_or_decrypt_file(&encrypt, false, file, &iv, &hash_seed);
        } else if !decrypt.is_empty() {
            encrypt_or_decrypt_file(&decrypt, true, file, &iv, &hash_seed);
        } else {
            if &*file == "-" {
                let mut buf: Vec<u8> = Vec::new();
                io::stdin().read(&mut buf).unwrap();
                let virtual_file = format!("malloc://{}", buf.len());
                if r_io::open_nomap(io, &virtual_file, 0, 0).is_null() {
                    let error = format!("Cannot open {}", virtual_file);
                    r_print::report(&error);
                }
                r_io::pwrite(io, 0, &buf);

            } else {
                if r_file::is_directory(file) {
                    r_print::report("Cannot hash directories");
                }
                if r_io::open_nomap(io, file, 0, 0).is_null() {
                    let error = format!("Cannot open {}", file);
                    r_print::report(&error);
                }
            }
            do_hash(file,
                    &algo,
                    io,
                    bsize,
                    &compare_bin,
                    &mut status,
                    &hash_seed);
        }
    }
}
