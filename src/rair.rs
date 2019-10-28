#[macro_use]
extern crate clap;
extern crate rio;
extern crate color_backtrace;
use rio::*;
use clap::{App, Arg};
use color_backtrace::{install_with_settings, Settings};
use std::num;

#[derive(Default)]
struct Core{
    io: RIO,
    cur: u64
}
impl Core {
    fn new() -> Self{
        let mut core: Core = Default::default();
        core.io = RIO::new();
        return core;
    }
}
fn str_to_num(n: &str)  -> Result<u64, num::ParseIntError> {
    match &*n[0..2].to_lowercase() {
        "0b" => return u64::from_str_radix(&n[2..], 2),
        "0x" => return u64::from_str_radix(&n[2..], 16),
        _ => (),
    }
    if n.chars().nth(0).unwrap() == '0' {
        return u64::from_str_radix(&n[1..], 8);
    }
    return u64::from_str_radix(n, 10);
}
fn main() {
    install_with_settings(Settings::new().message("Unrecoverable Error: "));
    let matches = App::new("rair")
                    .version(crate_version!())
                    .version_short("v")
                    .arg(Arg::with_name("Permission")
                            .help("File permision: Permission can be R or RW case insensitive, the default is R")
                            .short("p")
                            .long("perm")
                            .takes_value(true))
                    .arg(Arg::with_name("Paddr")
                            .help("Physical Base address")
                            .short("P")
                            .long("phy")
                            .takes_value(true))
                    .arg(Arg::with_name("File")
                            .help("Binary file to be loaded")
                            .takes_value(true)
                            .required(true))
                      .get_matches();
    let mut core = Core::new();
    let mut perm: IoMode = IoMode::READ;
    let mut paddr :u64 = 0;
    let uri = matches.value_of("File").unwrap();
    if let Some(p) = matches.value_of("Permission") {
        perm = Default::default() ;
        for c in p. to_lowercase().chars() {
            match c {
                'R' => perm |= IoMode::READ,
                'W' => perm |= IoMode::WRITE,
                _ => panic!("Unknown Permission: `{}`", c),
            }
        }
    }
    if let Some(addr) = matches.value_of("Paddr") {
        paddr = str_to_num(addr).unwrap_or_else(|e| panic!(e.to_string()));
    }
    core.io.open_at(uri, perm, paddr).unwrap_or_else(|e| panic!(e.to_string()));
    core.cur = paddr;
}
