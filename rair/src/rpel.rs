//! Read-Parse-Evaluate-Loop implementation.

use crate::{files::hist_file, lineformatter::LineFormatter};
use rair_core::Core;
use rair_eval::rair_eval;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::Editor;
use std::io::Write;
use std::process::exit;
use yansi::Paint;

pub fn prompt_read_parse_evaluate_loop(
    mut core: Core,
    mut editor: Editor<LineFormatter, FileHistory>,
) -> ! {
    loop {
        let prelude = &format!("[0x{:08x}]({})> ", core.get_loc(), core.mode);
        let (r, g, b) = core.env.read().get_color("color.2").unwrap();
        let input = editor.readline(&format!("{}", prelude.rgb(r, g, b)));
        match &input {
            Ok(line) => {
                editor.add_history_entry(line).unwrap();
                editor.save_history(&hist_file()).unwrap();
                rair_eval(&mut core, line);
            }
            Err(ReadlineError::Interrupted) => writeln!(core.stdout, "CTRL-C").unwrap(),
            Err(ReadlineError::Eof) => exit(0),
            Err(err) => writeln!(core.stdout, "Error: {err:?}").unwrap(),
        }
    }
}
