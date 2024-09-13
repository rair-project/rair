//! Initializers for rair cli.

use crate::lineformatter::LineFormatter;
use rair_core::Core;
use rustyline::{history::FileHistory, CompletionType, Config, EditMode, Editor};

pub fn init_editor_from_core(core: &mut Core) -> Editor<LineFormatter, FileHistory> {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let mut editor = Editor::with_config(config).unwrap();
    editor.set_helper(Some(LineFormatter::new(core.commands())));
    editor
}
