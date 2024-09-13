//! Read-Parse-Evaluate-Loop implementation.

use crate::{files::*, lineformatter::LineFormatter};
use rair_cmd::*;
use rair_core::{Core, Writer};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::Editor;
use std::fs::{File, OpenOptions};
use std::process::{Child, Command, Stdio};
use std::{io::prelude::*, io::Write, mem};
use yansi::Paint;

pub fn prompt_read_parse_evaluate_loop(
    mut core: Core,
    mut editor: Editor<LineFormatter, FileHistory>,
) -> ! {
    loop {
        let prelude = &format!("[0x{:08x}]({})> ", core.get_loc(), core.mode);
        let (r, g, b) = core.env.read().get_color("color.2").unwrap();
        let input = editor.readline(&format!("{}", Paint::rgb(r, g, b, prelude)));
        match &input {
            Ok(line) => {
                editor.add_history_entry(line).unwrap();
                editor.save_history(&hist_file()).unwrap();
                parse_evaluate(&mut core, line)
            }
            Err(ReadlineError::Interrupted) => writeln!(core.stdout, "CTRL-C").unwrap(),
            Err(ReadlineError::Eof) => std::process::exit(0),
            Err(err) => writeln!(core.stdout, "Error: {:?}", err).unwrap(),
        }
    }
}
fn parse_evaluate(core: &mut Core, line: &str) {
    let t = ParseTree::construct(line);
    if let Ok(tree) = t {
        evaluate(core, tree);
    } else {
        writeln!(core.stderr, "{}", t.err().unwrap()).unwrap();
    }
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
            Err(e) => return writeln!(core.stderr, "{}", e).unwrap(),
        }
    }
    // process redirections or pipes
    let mut stdout: Option<Writer> = None;
    let mut child: Option<Child> = None;
    match *cmd.red_pipe {
        RedPipe::Redirect(arg) => match create_redirect(core, *arg) {
            Ok(out) => stdout = Some(mem::replace(&mut core.stdout, out)),
            Err(e) => return writeln!(core.stderr, "{}", e).unwrap(),
        },
        RedPipe::RedirectCat(arg) => match create_redirect_cat(core, *arg) {
            Ok(out) => stdout = Some(mem::replace(&mut core.stdout, out)),
            Err(e) => return writeln!(core.stderr, "{}", e).unwrap(),
        },
        RedPipe::Pipe(arg) => match create_pipe(core, arg) {
            Ok((process, writer)) => {
                child = Some(process);
                stdout = Some(mem::replace(&mut core.stdout, writer));
            }
            Err(e) => return writeln!(core.stderr, "{}", e).unwrap(),
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
        let mut s = String::new();
        process
            .stdin
            .unwrap()
            .write_all(core.stdout.bytes_ref().unwrap())
            .unwrap();
        process.stdout.unwrap().read_to_string(&mut s).unwrap();
        writeln!(stdout.as_mut().unwrap(), "{}", s).unwrap();
    }
    // if we have a temporary stdout restore it
    if let Some(outstream) = stdout {
        core.stdout = outstream;
    }
}

fn create_redirect(core: &mut Core, arg: Argument) -> Result<Writer, String> {
    let file_name = eval_arg(core, arg)?;
    match File::create(file_name) {
        Ok(f) => Ok(Writer::new_write(Box::new(f))),
        Err(e) => Err(e.to_string()),
    }
}

fn create_redirect_cat(core: &mut Core, arg: Argument) -> Result<Writer, String> {
    let file_name = eval_arg(core, arg)?;
    match OpenOptions::new().append(true).open(file_name) {
        Ok(f) => Ok(Writer::new_write(Box::new(f))),
        Err(e) => Err(e.to_string()),
    }
}

fn create_pipe(
    core: &mut Core,
    unprocessed_args: Vec<Argument>,
) -> Result<(Child, Writer), String> {
    let mut args = Vec::with_capacity(unprocessed_args.len());
    for arg in unprocessed_args {
        args.push(eval_arg(core, arg)?);
    }
    match Command::new(&args[0])
        .args(&args[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Err(why) => Err(why.to_string()),
        Ok(process) => Ok((process, Writer::new_buf())),
    }
}

fn eval_arg(core: &mut Core, arg: Argument) -> Result<String, String> {
    match arg {
        Argument::Literal(s) => Ok(s),
        Argument::Err(e) => Err(e.to_string()),
        Argument::NonLiteral(c) => eval_non_literal_arg(core, c),
    }
}

fn eval_non_literal_arg(core: &mut Core, cmd: Cmd) -> Result<String, String> {
    // change stderr and stdout
    let mut stderr = Writer::new_buf();
    let mut stdout = Writer::new_buf();
    mem::swap(&mut core.stderr, &mut stderr);
    mem::swap(&mut core.stdout, &mut stdout);
    // run command
    run_cmd(core, cmd);
    // restore stderr and stdout
    mem::swap(&mut core.stderr, &mut stderr);
    mem::swap(&mut core.stdout, &mut stdout);

    let err = stderr.utf8_string().unwrap();
    if err.is_empty() {
        Err(err)
    } else {
        let out = stdout.utf8_string().unwrap();
        Ok(out)
    }
}
