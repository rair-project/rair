use core::mem::{replace, swap, take};
use rair_cmd::{Argument, Cmd, ParseTree, RedPipe};
use rair_core::{Core, Writer};
use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, Write},
    process::{Child, Command, Stdio},
};

pub fn rair_eval_no_out(core: &mut Core, line: &str) {
    let stdout = take(&mut core.stdout);
    let stderr = take(&mut core.stderr);
    rair_eval(core, line);
    core.stdout = stdout;
    core.stderr = stderr;
}
pub fn rair_eval(core: &mut Core, line: &str) {
    match ParseTree::construct(line) {
        Ok(tree) => evaluate(core, tree),
        Err(e) => writeln!(core.stderr, "{e}").unwrap(),
    }
}

fn evaluate(core: &mut Core, tree: ParseTree) {
    match tree {
        ParseTree::Help(help) => core.help(&help.command),
        ParseTree::Cmd(cmd) => run_cmd(core, cmd),
        ParseTree::HelpAll => core.help_all(),
        ParseTree::NewLine | ParseTree::Comment => (),
    }
}

fn run_cmd(core: &mut Core, cmd: Cmd) {
    let mut args = Vec::new();
    //process args
    for arg in cmd.args {
        match eval_arg(core, arg) {
            Ok(arg) => args.push(arg),
            Err(e) => return writeln!(core.stderr, "{e}").unwrap(),
        }
    }
    // process redirections or pipes
    let mut stdout: Option<Writer> = None;
    let mut child: Option<Child> = None;
    match *cmd.red_pipe {
        RedPipe::Redirect(arg) => match create_redirect(core, *arg) {
            Ok(out) => stdout = Some(replace(&mut core.stdout, out)),
            Err(e) => return writeln!(core.stderr, "{e}").unwrap(),
        },
        RedPipe::RedirectCat(arg) => match create_redirect_cat(core, *arg) {
            Ok(out) => stdout = Some(replace(&mut core.stdout, out)),
            Err(e) => return writeln!(core.stderr, "{e}").unwrap(),
        },
        RedPipe::Pipe(arg) => match create_pipe(core, arg) {
            Ok((process, writer)) => {
                child = Some(process);
                stdout = Some(replace(&mut core.stdout, writer));
            }
            Err(e) => return writeln!(core.stderr, "{e}").unwrap(),
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
        writeln!(stdout.as_mut().unwrap(), "{s}").unwrap();
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
        _ => unreachable!(),
    }
}

fn eval_non_literal_arg(core: &mut Core, cmd: Cmd) -> Result<String, String> {
    // change stderr and stdout
    let mut stderr = Writer::new_buf();
    let mut stdout = Writer::new_buf();
    swap(&mut core.stderr, &mut stderr);
    swap(&mut core.stdout, &mut stdout);
    // run command
    run_cmd(core, cmd);
    // restore stderr and stdout
    swap(&mut core.stderr, &mut stderr);
    swap(&mut core.stdout, &mut stdout);

    let err = stderr.utf8_string().unwrap();
    if err.is_empty() {
        Err(err)
    } else {
        let out = stdout.utf8_string().unwrap();
        Ok(out)
    }
}

#[cfg(test)]
mod test;
