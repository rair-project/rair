#![warn(clippy::cargo)]
#![allow(clippy::needless_return)]

#[macro_use]
extern crate clap;

extern crate app_dirs;
extern crate color_backtrace;
extern crate rcmd;
extern crate rio;
extern crate rustyline;
use app_dirs::*;
use clap::{App, Arg};
use color_backtrace::{install_with_settings, Settings};
use rcmd::ParseTree;
use rio::*;
use rustyline::{error::ReadlineError, Editor};
use std::num;
use std::path::PathBuf;

struct Core {
    io: RIO,
    rl: Editor<()>,
    cur: u64,
    app_info: AppInfo,
}
impl Core {
    fn new() -> Self {
        let mut core = Core {
            io: RIO::new(),
            cur: 0,
            rl: Editor::<()>::new(),
            app_info: AppInfo { name: "rair", author: "RairDevs" },
        };
        drop(core.rl.load_history(&core.hist_file()));
        return core;
    }
    fn hist_file(&self) -> PathBuf {
        let mut history = app_dir(AppDataType::UserData, &self.app_info, "/").unwrap();
        history.push("history");
        return history;
    }
}

fn str_to_num(n: &str) -> Result<u64, num::ParseIntError> {
    if n.len() >= 2 {
        match &*n[0..2].to_lowercase() {
            "0b" => return u64::from_str_radix(&n[2..], 2),
            "0x" => return u64::from_str_radix(&n[2..], 16),
            _ => (),
        }
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
        .arg(
            Arg::with_name("Permission")
                .help("File permision: Permission can be R or RW case insensitive, the default is R")
                .short("p")
                .long("perm")
                .takes_value(true),
        )
        .arg(Arg::with_name("Paddr").help("Physical Base address").short("P").long("phy").takes_value(true))
        .arg(Arg::with_name("File").help("Binary file to be loaded").takes_value(true).required(true))
        .get_matches();
    let mut core = Core::new();
    let mut perm: IoMode = IoMode::READ;
    let mut paddr: u64 = 0;
    let uri = matches.value_of("File").unwrap();
    if let Some(p) = matches.value_of("Permission") {
        perm = Default::default();
        for c in p.to_lowercase().chars() {
            match c {
                'r' => perm |= IoMode::READ,
                'w' => perm |= IoMode::WRITE,
                _ => panic!("Unknown Permission: `{}`", c),
            }
        }
    }
    if let Some(addr) = matches.value_of("Paddr") {
        paddr = str_to_num(addr).unwrap_or_else(|e| panic!(e.to_string()));
    }
    core.io.open_at(uri, perm, paddr).unwrap_or_else(|e| panic!(e.to_string()));
    core.cur = paddr;
    loop {
        repl_inners(&mut core);
    }
}

fn repl_inners(core: &mut Core) {
    let input = core.rl.readline(&format!("[0x{:08x}]> ", core.cur));
    match &input {
        Ok(line) => {
            core.rl.add_history_entry(line);
            let t = ParseTree::construct(line);
            if let Ok(tree) = t {
                println!("{:#?}", tree);
            } else {
                println!("{}", t.err().unwrap().to_string());
            }
        }
        Err(ReadlineError::Interrupted) => {
            println!("CTRL-C");
        }
        Err(ReadlineError::Eof) => {
            std::process::exit(0);
        }
        Err(err) => {
            println!("Error: {:?}", err);
        }
    }
    core.rl.save_history(&core.hist_file()).unwrap();
}
