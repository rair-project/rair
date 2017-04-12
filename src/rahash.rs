/*
 *  rahash.rs -- block based hashing utility
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

//XXX bug in xor encryption scheme ...
// rahash -e xor -s BAAA  -S 4
// this crashes because encryption key is shorter than text
// it should be reused aka otp
extern crate getopts;
extern crate libc;
extern crate r_crypto;
extern crate r_hash;
extern crate r_io;
extern crate r_util;
//TODO get rid of rustc_serialize
extern crate rustc_serialize;

mod version;

use getopts::Options;
use libc::*;
use rustc_serialize::hex::FromHex;
use std::env;
use std::process;
use std::io::{self, Read, Write};
use std::ffi::{CStr, CString}; //TODO get rid of me
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
        //TODO make me safe
        let name: String = unsafe {
            CStr::from_ptr(r_hash::r_hash_name(bits))
                .to_string_lossy()
                .into_owned()
        };
        if !name.is_empty() {
            println!("  {}", name);
        }
    }
    println!("Available Encoders/Decoders:");
    println!("  base64\n  base91\n  punycode\nAvailable Crypto Algos:");
    for i in 0..64 {
        let bits: u64 = 1 << i;
        let name: String = unsafe {
            CStr::from_ptr(r_crypto::r_crypto_name(bits))
                .to_string_lossy()
                .into_owned()
        };
        if !name.is_empty() {
            println!("  {}", name);
        }
    }
}

fn report(error: &str) -> ! {
    writeln!(&mut std::io::stderr(), "{}", error).unwrap();
    process::exit(1);
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
fn do_hash_print(ctx: &r_hash::RHash,
                 algo: u64,
                 little_endian: bool,
                 len: usize,
                 status: &Status) {
    let hname: String = unsafe {
        CStr::from_ptr(r_hash::r_hash_name(algo))
            .to_string_lossy()
            .into_owned()
    };
    let mut c: Vec<u8> = (*ctx).digest.to_vec();
    c.drain(len..);
    match status.format {
        OutputFormat::None => {
            if !status.quiet {
                print!("0x{:08x}-0x{:08x} {}: ", status.from, status.to - 1, hname);
            }
            do_hash_hexprint(&c, little_endian);
            println!("");
        }
        OutputFormat::Json => {
            print!("{{\"name\":\"{}\",\"hash\":\"", hname);
            do_hash_hexprint(&c, little_endian);
            println!("\"}}");
        }
        OutputFormat::Command => {
            print!("e file.{}=", hname);
            do_hash_hexprint(&c, little_endian);
            println!("");
        }
        OutputFormat::Ssh => {
            let art: String = unsafe {
                CStr::from_ptr(r_util::r_print_randomart(c.as_ptr(), c.len(), status.from))
                    .to_string_lossy()
                    .into_owned()
            };
            println!("{}\n{}", hname, art);
        }
    };
}

fn do_hash_internal(ctx: &r_hash::RHash,
                    algo: u64,
                    buf: &[u8],
                    print: bool,
                    little_endian: bool,
                    status: &Status,
                    s: &r_hash::RHashSeed) {
    let dlen: usize = unsafe { r_hash::r_hash_calculate(ctx, algo, buf.as_ptr(), buf.len()) };
    if dlen == 0 {
        return;
    }
    if !print {
        return;
    }
    if algo == r_hash::R_HASH_ENTROPY {
        let e: f64 = unsafe { r_hash::r_hash_entropy(buf.as_ptr(), buf.len()) };
        if status.format == OutputFormat::None {
            print!("0x{:08x}-0x{:08x} {:.10}", status.from, status.to - 1, e);
            unsafe { r_util::r_print_progressbar(ptr::null(), (12.5 * e) as i32, 60) };
            println!("");
        } else {
            //TODO that doesn't look either radare2 commands or json tbh
            println!("entropy: {:.10}", e);
        }
    } else {
        if status.iterations > 0 {
            let cseed = r_hash::CRHashSeed {
                prefix: s.prefix,
                len: s.buf.len(),
                buf: s.buf.clone().as_ptr(),
            };
            unsafe { r_hash::r_hash_do_spice(ctx, algo, status.iterations, &cseed) };
        }
        do_hash_print(ctx, algo, little_endian, dlen, status);

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
           io: *mut c_void,
           mut bsize: usize,
           little_endian: bool,
           compare: &[u8],
           mut status: &mut Status,
           s: &r_hash::RHashSeed) {
    let algobit =
        unsafe { r_hash::r_hash_name_to_bits(CString::new(&*algo.clone()).unwrap().as_ptr()) };
    if algobit == r_hash::R_HASH_NONE {
        report("Invalid hashing algorithm ");
    }
    let fsize = unsafe { r_io::r_io_size(io) };
    if bsize == 0 || bsize > fsize {
        bsize = fsize;
    }
    if status.to == 0 {
        status.to = fsize;
    }
    if status.from > status.to {
        report("Invalid -f -t range");
    }
    let ctx = unsafe { &*r_hash::r_hash_new(true, algobit) };
    if status.format == OutputFormat::Json {
        print!("[");
    }
    if status.incremental {
        let mut i = 1;
        let mut first: bool = true;
        while i < r_hash::R_HASH_ALL {
            let hashbit = algobit & i;
            if hashbit != 0 {
                let dlen = unsafe { r_hash::r_hash_size(algobit) };
                unsafe { r_hash::r_hash_do_begin(ctx, i) };
                if status.format == OutputFormat::Json {
                    if first {
                        first = false;
                    } else {
                        print!(",");
                    }
                }
                if s.prefix & !s.buf.is_empty() {
                    do_hash_internal(ctx, hashbit, &s.buf, false, little_endian, status, s);
                }
                let mut j = status.from;
                while j < status.to {
                    let nsize: usize = if j + bsize > status.to {
                        status.to - j
                    } else {
                        bsize
                    };
                    let buf: Vec<u8> = vec![0; nsize];
                    unsafe { r_io::r_io_pread(io, j as u64, buf.as_ptr(), nsize) };
                    do_hash_internal(ctx, hashbit, &buf, false, little_endian, status, s);
                    j += bsize;
                }
                if s.prefix & !s.buf.is_empty() {
                    do_hash_internal(ctx, hashbit, &s.buf, false, little_endian, status, s);
                }
                unsafe { r_hash::r_hash_do_end(ctx, i) };
                if status.iterations > 0 {
                    let cseed = r_hash::CRHashSeed {
                        prefix: s.prefix,
                        len: s.buf.len(),
                        buf: s.buf.clone().as_ptr(),
                    };
                    unsafe { r_hash::r_hash_do_spice(ctx, i, status.iterations, &cseed) };
                }
                if !status.quiet && status.format != OutputFormat::Json {
                    print!("{} ", file);
                }
                do_hash_print(ctx, i, little_endian, dlen, status);
            }
            i <<= 1;
        }
    } else {
        if !s.buf.is_empty() {
            report("Seed cannot be used on per-block hashing.");
        }

        let mut i = 1;
        while i < r_hash::R_HASH_ALL {
            let hashbit = algobit & i;
            if hashbit != 0 {
                let mut j = status.from;
                let mut status_c = status.clone();
                while j < status.to {
                    let nsize: usize = if j + bsize < fsize { bsize } else { fsize - j };
                    let buf: Vec<u8> = vec![0; nsize];
                    unsafe { r_io::r_io_pread(io, j as u64, buf.as_ptr(), nsize) };
                    status_c.from = j;
                    status_c.to = j + bsize;
                    if status_c.to > fsize {
                        status_c.to = fsize;
                    }
                    do_hash_internal(ctx, hashbit, &buf, true, little_endian, &status_c, s);
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
        let hash_size = unsafe { r_hash::r_hash_size(algobit) };
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
                Err(why) => report(&why.to_string()),
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
            report("Decryption key is not defined. Use -S [key]");
        } else {
            report("Encryption key is not defined. Use -S [key]");
        }
    }
    let cry = unsafe { r_crypto::r_crypto_new() };
    if !unsafe { r_crypto::r_crypto_use(cry, CString::new(algo.clone()).unwrap().as_ptr()) } {
        if is_decryption {
            let err = format!("Unknown decryption algorithm '{}'", algo);
            report(&*err);
        } else {
            let err = format!("Unknown encryption algorithm '{}'", algo);
            report(&*err);
        }
    }
    if !unsafe { r_crypto::r_crypto_set_key(cry, s.buf.as_ptr(), s.buf.len(), 0, is_decryption) } {
        report("Invalid key");
    }
    if !iv.is_empty() && !unsafe { r_crypto::r_crypto_set_iv(cry, iv.as_ptr(), iv.len()) } {
        report("Invalid initialization vector");
    }
    unsafe { r_crypto::r_crypto_update(cry, buf.as_ptr(), buf.len()) };
    unsafe { r_crypto::r_crypto_final(cry, ptr::null(), 0) };
    let result = r_crypto::get_output(cry);
    std::io::stdout().write(&result).unwrap();
}

fn do_hash_seed(mut seed: String) -> r_hash::RHashSeed {
    let mut r_hash_seed = r_hash::RHashSeed::new();
    if seed.is_empty() {
        return r_hash_seed;
    }
    if &seed == "-" {
        io::stdin().read(&mut r_hash_seed.buf).unwrap();
        return r_hash_seed;
    }
    if &seed[0..1] == "^" {
        r_hash_seed.prefix = true;
        seed.remove(0);
    } else {
        r_hash_seed.prefix = false;
    }
    if &seed[0..2] == "S:" {
        seed.drain(0..2);
        r_hash_seed.buf.extend(seed.as_bytes());
    } else {
        r_hash_seed.buf = match (*seed).from_hex() {
            Ok(buf) => buf,
            Err(why) => report(&(why.to_string())),
        }
    }
    r_hash_seed
}

fn is_power_of_two(x: u64) -> bool {
    (x != 0) && (x & (x - 1)) == 0
}

fn main() {
    //TODO option n that that I dont really know what is the high level description of its
    //behaviour
    let mut status: Status = Status {
        quiet: false,
        iterations: 0,
        format: OutputFormat::None,
        incremental: true,
        from: 0,
        to: 0,
    };
    let mut hashstr = String::new();
    let mut compare_str = String::new();
    let mut decrypt: String = String::new();
    let mut encrypt: String = String::new();
    let mut ivseed: String = String::new();
    let mut ishex = false;
    let mut bsize: usize = 0;
    let mut algo = "sha256".to_owned();
    let args: Vec<String> = env::args().collect();
    let mut little_endian = false;
    let mut opts = Options::new();
    let mut seed: String = String::new();
    let mut iv: Vec<u8> = Vec::new();
    let mut hash: Vec<u8> = Vec::new();
    let mut compare_bin: Vec<u8> = Vec::new();
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
        Err(f) => report(&f.to_string()),
    };
    if matches.opt_present("h") {
        let program = args[0].clone();
        let help = format!("Usage: {} [-rBhlkvje] [-b S] [-a A] [-c H] [-e A] \
                           [-s S] [-f O] [-t O] [file] ...",
                           program);
        print!("{}", opts.usage(&help));
        return;
    }
    if matches.opt_present("l") {
        algolist();
        return;
    }
    if matches.opt_present("v") {
        let program = &args[0];
        version::blob_version(program);
        return;
    }
    if matches.opt_present("q") {
        status.quiet = true;
    }
    if matches.opt_present("i") {
        let tmp = matches.opt_str("i").unwrap();
        match (&tmp).parse() {
            Ok(m) => status.iterations = m,
            Err(f) => report(&f.to_string()),
        }
    }
    if matches.opt_present("j") {
        match status.format {
            OutputFormat::None => status.format = OutputFormat::Json,
            _ => {
                report("`-j`, `-r` and `-k` are not compatiable, you can not \
                        use any two of them at the same time")
            }
        }
    }
    if matches.opt_present("r") {
        match status.format {
            OutputFormat::None => status.format = OutputFormat::Command,
            _ => {
                report("`-j`, `-r` and `-k` are not compatiable, you can not \
                        use any two of them at the same time")
            }
        }
    }
    if matches.opt_present("k") {
        match status.format {
            OutputFormat::None => status.format = OutputFormat::Ssh,
            _ => {
                report("`-j`, `-r` and `-k` are not compatiable, you can not \
                        use any two of them at the same time")
            }
        }
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
    if matches.opt_present("E") {
        little_endian = true;
    }
    if matches.opt_present("a") {
        algo = matches.opt_str("a").unwrap();
    }
    if matches.opt_present("B") {
        status.incremental = false;
    }
    if matches.opt_present("b") {
        let tmp = matches.opt_str("b").unwrap();
        //TODO make me safe
        let cstring = CString::new(tmp).unwrap();
        bsize = unsafe { r_util::r_num_math(ptr::null(), cstring.as_ptr()) } as usize;

    }
    if matches.opt_present("f") {
        let tmp = matches.opt_str("f").unwrap();
        //TODO make me safe
        let cstring = CString::new(tmp).unwrap();
        status.from = unsafe { r_util::r_num_math(ptr::null(), cstring.as_ptr()) } as usize;
    }
    if matches.opt_present("t") {
        let tmp = matches.opt_str("t").unwrap();
        //TODO make me safe
        let cstring = CString::new(tmp).unwrap();
        status.to = unsafe { r_util::r_num_math(ptr::null(), cstring.as_ptr()) } as usize + 1;
    }
    if matches.opt_present("s") && matches.opt_present("x") {
        report(" -s and -x are not compatiable, you can not \
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
        report("Option -c incompatible with -b and -B options.");
    }
    if matches.opt_present("c") {
        compare_str = matches.opt_str("c").unwrap();
    }
    if matches.opt_present("e") && matches.opt_present("d") {
        report("Option -e and -d are incompatible with each other.")
    }
    if !compare_str.is_empty() {
        let algobit: u64;
        if &encrypt == "base64" || &encrypt == "base91" || &decrypt == "base64" ||
           &decrypt == "base91" {
            report("Option -c incompatible with -E base64, -E base91, -D base64 or \
                   -D base91 options.");
        }
        //TODO make me safe
        let cstring = CString::new(&*algo).unwrap();
        algobit = unsafe { r_hash::r_hash_name_to_bits(cstring.as_ptr()) };
        //TODO heavily document how r_hash_name_to_bits works
        // the myth says it returns a number that is power of 2 if only 1 algo is used
        if !is_power_of_two(algobit) {
            report("Option -c incompatible with multiple algorithms in -a.");
        }
        compare_bin = match (*compare_str).from_hex() {
            Err(why) => report(&why.to_string()),
            Ok(x) => x,
        };
        //TODO make me safe
        if compare_bin.len() != unsafe { r_hash::r_hash_size(algobit) } {
            let err_msg = format!("rahash2: Given -c hash has {} bytes but the \
                selected algorithm returns {} bytes.",
                                  compare_bin.len(),
                                  unsafe { r_hash::r_hash_size(algobit) }); // TODO make me safe
            report(&err_msg);
        }
    }
    if status.to != 0 && status.from >= status.to {
        report("Invalid -f or -t offsets\n");
    }
    if !ivseed.is_empty() {
        if &ivseed[0..2] == "s:" {
            iv.extend(ivseed[2..].as_bytes());
        } else {
            iv = match (*ivseed).from_hex() {
                Err(why) => report(&why.to_string()),
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
                Err(why) => report(&why.to_string()),
                Ok(x) => x,
            }
        } else {
            hash.extend(hashstr.as_bytes());
        }
        if status.from >= hash.len() {
            report("-f value is greater than hash length");
        }
        if status.to > hash.len() {
            report("-t value is greater than hash length");
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
            //TODO make me safe
            let cstring = CString::new(&*algo).unwrap();
            let algobit: u64 = unsafe { r_hash::r_hash_name_to_bits(cstring.as_ptr()) };
            if algobit == 0 {
                report("Invalid algorithm. See -E and -D");
            }
            //TODO THIS SUCKS SHOULD BE IMPROVED FOR SURE
            let mut i = 1;
            while i < r_hash::R_HASH_ALL {
                let hashbit = algobit & i;
                if hashbit != 0 {
                    //TODO make me safe
                    let ctx = unsafe { &*r_hash::r_hash_new(true, hashbit) };
                    status.from = 0;
                    status.to = full_str.len();
                    do_hash_internal(ctx,
                                     hashbit,
                                     &full_str,
                                     true,
                                     little_endian,
                                     &status,
                                     &hash_seed);
                    if !compare_bin.is_empty() {
                        //TODO make me safe
                        let hash_size = unsafe { r_hash::r_hash_size(algobit) };
                        compare_hashes(ctx, &compare_bin, hash_size);
                    }
                }
                i <<= 1;
            }
            return;
        }
    }
    for file in &matches.free {
        let io: *mut c_void = unsafe { r_io::r_io_new() };
        if !encrypt.is_empty() {
            encrypt_or_decrypt_file(&encrypt, false, file, &iv, &hash_seed);
        } else if !decrypt.is_empty() {
            encrypt_or_decrypt_file(&decrypt, true, file, &iv, &hash_seed);
        } else {
            if &*file == "-" {
                let mut buf: Vec<u8> = Vec::new();
                io::stdin().read(&mut buf).unwrap();
                let virtual_file = format!("malloc://{}", buf.len());
                let filecstring = CString::new(virtual_file.clone()).unwrap();
                //TODO make me safe
                if unsafe { r_io::r_io_open_nomap(io, filecstring.as_ptr(), 0, 0) }.is_null() {
                    let error = format!("Cannot open {}", virtual_file);
                    report(&error);
                }
                //TODO make me safe
                unsafe { r_io::r_io_pwrite(io, 0, buf.as_ptr(), buf.len()) };

            } else {
                //TODO make me safe
                let cstring = CString::new(file.to_owned()).unwrap();
                if unsafe { r_util::r_file_is_directory(cstring.as_ptr()) } {
                    report("Cannot hash directories");
                }
                //TODO make me safe
                if unsafe { r_io::r_io_open_nomap(io, cstring.as_ptr(), 0, 0) }.is_null() {
                    let error = format!("Cannot open {}", file);
                    report(&error);
                }
            }
            do_hash(file,
                    &algo,
                    io,
                    bsize,
                    little_endian,
                    &compare_bin,
                    &mut status,
                    &hash_seed);
        }
    }
}
