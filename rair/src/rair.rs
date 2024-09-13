//! rair CLI.

mod cli;
mod files;
mod init;
mod lineformatter;
mod rpel;

use cli::Args;
use init::*;
use rair_core::{panic_msg, Core, Writer};
use rpel::*;
use std::mem;

fn main() {
    let mut core = Core::new();
    let editor = init_editor_from_core(&mut core);
    let args = Args::parse().unwrap_or_else(|e| panic_msg(&mut core, &e, ""));
    match args {
        Args::Proj(proj) => {
            let stderr = mem::replace(&mut core.stderr, Writer::new_buf());
            core.run("load", &[proj]);
            let err_buf = mem::replace(&mut core.stderr, stderr)
                .utf8_string()
                .unwrap();
            if !err_buf.is_empty() {
                panic_msg(&mut core, "", &err_buf);
            }
        }
        Args::File { uri, base, perms } => {
            core.io
                .open_at(&uri, perms, base)
                .unwrap_or_else(|e| panic_msg(&mut core, &e.to_string(), ""));
            core.set_loc(base);
        }
    }
    prompt_read_parse_evaluate_loop(core, editor);
}
