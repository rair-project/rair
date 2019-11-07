#![warn(clippy::cargo)]
#![allow(clippy::needless_return)]

#[macro_use]
extern crate clap;
extern crate app_dirs;
extern crate color_backtrace;
extern crate rcmd;
extern crate rcore;
extern crate rio;
extern crate rtrees;
extern crate rustyline;

use clap::{App, Arg};
use color_backtrace::{install_with_settings, Settings};
use rcmd::*;
use rcore::*;
use rio::*;
use rustyline::error::ReadlineError;
use std::fs::{File, OpenOptions};
use std::process::{Child, Command, Stdio};
use std::{io::prelude::*, io::Write, mem, num};

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
    core.set_loc(paddr);
    loop {
        repl_inners(&mut core);
    }
}

fn repl_inners(core: &mut Core) {
    let input = core.rl.readline(&format!("[0x{:08x}]> ", core.get_loc()));
    match &input {
        Ok(line) => {
            core.rl.add_history_entry(line);
            let t = ParseTree::construct(line);
            if let Ok(tree) = t {
                evaluate(core, tree);
            } else {
                writeln!(core.stderr, "{}", t.err().unwrap().to_string()).unwrap();
            }
        }
        Err(ReadlineError::Interrupted) => {
            writeln!(core.stdout, "CTRL-C").unwrap();
        }
        Err(ReadlineError::Eof) => {
            std::process::exit(0);
        }
        Err(err) => {
            writeln!(core.stdout, "Error: {:?}", err).unwrap();
        }
    }
    core.rl.save_history(&core.hist_file()).unwrap();
}

fn evaluate(core: &mut Core, tree: ParseTree) {
    match tree {
        ParseTree::Help(help) => core.help(&help.command),
        ParseTree::Cmd(cmd) => run_cmd(core, cmd),
        ParseTree::NewLine => (),
        ParseTree::Comment => (),
    }
}

fn run_cmd(core: &mut Core, cmd: Cmd) {
    let mut args = Vec::new();
    //process args
    for arg in cmd.args {
        match eval_arg(core, arg) {
            Ok(arg) => args.push(arg),
            Err(e) => {
                writeln!(core.stderr, "{}", e).unwrap();
                return;
            }
        }
    }
    // process redirections or pipes
    let mut stdout: Option<Writer> = None;
    let mut child: Option<Child> = None;
    match *cmd.red_pipe {
        RedPipe::Redirect(arg) => match create_redirect(core, *arg) {
            Ok(out) => stdout = Some(mem::replace(&mut core.stdout, out)),
            Err(e) => {
                writeln!(core.stderr, "{}", e).unwrap();
                return;
            }
        },
        RedPipe::RedirectCat(arg) => match create_redirect_cat(core, *arg) {
            Ok(out) => stdout = Some(mem::replace(&mut core.stdout, out)),
            Err(e) => {
                writeln!(core.stderr, "{}", e).unwrap();
                return;
            }
        },
        RedPipe::Pipe(arg) => match create_pipe(core, *arg) {
            Ok((process, writer)) => {
                child = Some(process);
                stdout = Some(mem::replace(&mut core.stdout, writer));
            }
            Err(e) => {
                writeln!(core.stderr, "{}", e).unwrap();
                return;
            }
        },
        RedPipe::None => (),
    }
    // execute
    match cmd.loc {
        Some(at) => core.run_at(&cmd.command, &args, at),
        None => core.run(&cmd.command, &args),
    }
    //if we have a pipe feed into the pipe ..
    if let Some(process) = child {
        if let Writer::Bytes(b) = &mut core.stdout {
            let mut s = String::new();
            process.stdin.unwrap().write_all(&b).unwrap();
            process.stdout.unwrap().read_to_string(&mut s).unwrap();
            writeln!(stdout.as_mut().unwrap(), "{}", s).unwrap();
        }
    }
    // if we have a temporary stdout restore it
    if let Some(outstream) = stdout {
        mem::replace(&mut core.stdout, outstream);
    }
}

fn create_redirect(core: &mut Core, arg: Argument) -> Result<Writer, String> {
    let file_name = eval_arg(core, arg)?;
    match File::create(file_name) {
        Ok(f) => return Ok(Writer::Write(Box::new(f))),
        Err(e) => return Err(e.to_string()),
    }
}

fn create_redirect_cat(core: &mut Core, arg: Argument) -> Result<Writer, String> {
    let file_name = eval_arg(core, arg)?;
    match OpenOptions::new().write(true).append(true).open(file_name) {
        Ok(f) => return Ok(Writer::Write(Box::new(f))),
        Err(e) => return Err(e.to_string()),
    }
}

fn create_pipe(core: &mut Core, arg: Argument) -> Result<(Child, Writer), String> {
    let process_name = eval_arg(core, arg)?;
    match Command::new(process_name).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn() {
        Err(why) => return Err(why.to_string()),
        Ok(process) => return Ok((process, Writer::Bytes(Vec::new()))),
    };
}

fn eval_arg(core: &mut Core, arg: Argument) -> Result<String, String> {
    match arg {
        Argument::Literal(s) => return Ok(s),
        Argument::Err(e) => return Err(e.to_string()),
        Argument::NonLiteral(c) => return eval_non_literal_arg(core, c),
    }
}
fn eval_non_literal_arg(core: &mut Core, cmd: Cmd) -> Result<String, String> {
    // change stderr and stdout
    let mut stderr = Writer::Bytes(Vec::new());
    let mut stdout = Writer::Bytes(Vec::new());
    mem::swap(&mut core.stderr, &mut stderr);
    mem::swap(&mut core.stdout, &mut stdout);
    // run command
    run_cmd(core, cmd);
    // restore stderr and stdout
    mem::swap(&mut core.stderr, &mut stderr);
    mem::swap(&mut core.stdout, &mut stdout);

    if let Writer::Bytes(err) = stderr {
        if err.is_empty() {
            return Err(String::from_utf8(err).unwrap());
        } else {
            if let Writer::Bytes(out) = stdout {
                return Ok(String::from_utf8(out).unwrap());
            } else {
                return Err("BUG: Expected Bytes based Writerin STDOUT".to_string());
            }
        }
    } else {
        return Err("BUG: Expected Bytes based Writer in STDERR".to_string());
    }
}
