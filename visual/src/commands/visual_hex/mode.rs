use super::{command::Command, editor::Editor, selector::Selector};

#[derive(Default)]
pub enum Mode {
    #[default]
    View,
    Edit(Editor),
    Command(Command),
    Select(Selector),
}

impl Mode {
    pub fn new_command() -> Self {
        Mode::Command(Default::default())
    }
}
