//! rair CLI.

mod files;
mod init;
mod lineformatter;
mod rpel;

use clap::{crate_version, load_yaml, App, ArgMatches};
use init::*;
use rair_core::{panic_msg, str_to_num, Core, Writer};
use rair_io::IoMode;
use rpel::*;
use std::mem;

enum Matches {
    Project,
    File,
}
fn check_matches(matches: &ArgMatches) -> Result<Matches, &'static str> {
    let paddr = matches.value_of("Base");
    let uri = matches.value_of("File");
    let perm = matches.value_of("Permission");
    let proj = matches.value_of("Project");
    if !(proj.is_some() ^ uri.is_some()) {
        return Err("You must open either a binary file or Project file, but not both");
    }
    if proj.is_some() && paddr.is_some() {
        return Err("You cannot set Base address when opening a project");
    }
    if proj.is_some() && perm.is_some() {
        return Err("You cannot set permissions when opening a project");
    }
    if proj.is_some() {
        return Ok(Matches::Project);
    } else {
        return Ok(Matches::File);
    }
}
fn match_file(core: &mut Core, matches: &ArgMatches) {
    let paddr = matches
        .value_of("Base")
        .map(|addr| str_to_num(addr).unwrap_or_else(|e| panic_msg(core, &e.to_string(), "")));
    let uri = matches.value_of("File").unwrap();
    let mut perm: IoMode = IoMode::READ;
    if let Some(p) = matches.value_of("Permission") {
        perm = Default::default();
        for c in p.to_lowercase().chars() {
            match c {
                'r' => perm |= IoMode::READ,
                'w' => perm |= IoMode::WRITE,
                'c' => perm |= IoMode::COW,
                _ => panic_msg(core, &format!("Unknown Permission: `{}`", c), ""),
            }
        }
    }
    if let Some(paddr) = paddr {
        core.io
            .open_at(uri, perm, paddr)
            .unwrap_or_else(|e| panic_msg(core, &e.to_string(), ""));
        core.set_loc(paddr);
    } else {
        let hndl = core
            .io
            .open(uri, perm)
            .unwrap_or_else(|e| panic_msg(core, &e.to_string(), ""));
        core.set_loc(core.io.hndl_to_desc(hndl).unwrap().paddr_base());
    }
}

fn match_project(core: &mut Core, matches: &ArgMatches) {
    let project = matches.value_of("Project").unwrap();
    let stderr = mem::replace(&mut core.stderr, Writer::new_buf());
    core.run("load", &[project.to_string()]);
    let err_buf = mem::replace(&mut core.stderr, stderr)
        .utf8_string()
        .unwrap();
    if !err_buf.is_empty() {
        panic_msg(core, "", &err_buf);
    }
}
fn main() {
    let mut core = Core::new();
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml)
        .version(crate_version!())
        .version_short("v")
        .get_matches();
    let editor = init_editor_from_core(&mut core);
    let match_type = check_matches(&matches).unwrap_or_else(|e| panic_msg(&mut core, e, ""));
    match match_type {
        Matches::File => match_file(&mut core, &matches),
        Matches::Project => match_project(&mut core, &matches),
    }
    prompt_read_parse_evaluate_loop(core, editor);
}
