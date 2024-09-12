//! Initializers for rair cli.

use crate::lineformatter::LineFormatter;
use rair_core::Core;
use rustyline::{CompletionType, Config, EditMode, Editor, OutputStreamType};
pub fn init_editor_from_core(core: &mut Core) -> Editor<LineFormatter> {
    let config = Config::builder()
        .completion_type(CompletionType::Circular)
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build();
    let mut editor = Editor::with_config(config);
    editor.set_helper(Some(LineFormatter::new(core.commands())));
    editor
}
