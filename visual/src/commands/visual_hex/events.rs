use rair_core::Core;
use rair_eval::rair_eval_no_out;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::visual_hex::VisualHexEnv;

use super::{editor::Editor, mode::Mode, selector::Selector, NextAction, VisualHex};

impl VisualHex {
    fn handle_input_view(
        &mut self,
        input: KeyEvent,
        core: &mut Core,
        env: &VisualHexEnv,
    ) -> NextAction {
        match input.code {
            KeyCode::Down => core.set_loc(core.get_loc().saturating_add(16)),
            KeyCode::Up => core.set_loc(core.get_loc().saturating_sub(16)),
            KeyCode::Right => core.set_loc(core.get_loc().saturating_add(1)),
            KeyCode::Left => core.set_loc(core.get_loc().saturating_sub(1)),
            KeyCode::Char(c) => {
                if c == env.edit_mode_button {
                    self.mode = Mode::Edit(Editor::new(core));
                } else if c == env.command_mode_button {
                    self.mode = Mode::new_command();
                } else if c == env.select_mode_button {
                    self.mode = Mode::Select(Selector::new(core));
                } else if c == env.quit_buton {
                    return NextAction::Break;
                }
            }
            _ => (),
        }
        NextAction::Continue
    }

    pub(super) fn handle_input(
        &mut self,
        input: KeyEvent,
        core: &mut Core,
        env: &VisualHexEnv,
    ) -> NextAction {
        match &mut self.mode {
            Mode::View => return self.handle_input_view(input, core, env),
            Mode::Edit(editor) => match input.code {
                KeyCode::Esc => self.mode = Mode::View,
                KeyCode::Down => editor.down(core, self.rendered_bytes),
                KeyCode::Up => editor.up(core),
                KeyCode::Right => editor.right(core, self.rendered_bytes),
                KeyCode::Left => editor.left(core),
                KeyCode::Tab => editor.switch_mode(),
                KeyCode::Char(c) => {
                    editor.write_char(core, c);
                    editor.right(core, self.rendered_bytes);
                }
                _ => (),
            },
            Mode::Command(cmd) => match input.code {
                KeyCode::Esc => self.mode = Mode::View,
                KeyCode::Left => cmd.move_cursor_left(),
                KeyCode::Right => cmd.move_cursor_right(),
                KeyCode::Backspace => cmd.delete_char(),
                KeyCode::Delete => {
                    cmd.move_cursor_right();
                    cmd.delete_char();
                }
                KeyCode::Char(c) => {
                    cmd.enter_char(c);
                }
                KeyCode::Enter => {
                    let cmd = cmd.get_str();
                    rair_eval_no_out(core, cmd);
                    self.mode = Mode::View;
                }
                _ => (),
            },
            Mode::Select(selector) => match input.code {
                KeyCode::Esc => {
                    if selector.is_active() {
                        selector.deactivate()
                    } else {
                        self.mode = Mode::View;
                    }
                }
                KeyCode::Right => selector.right(core, self.rendered_bytes),
                KeyCode::Left => selector.left(core),
                KeyCode::Up => selector.up(core),
                KeyCode::Down => selector.down(core, self.rendered_bytes),
                KeyCode::Char(c) => {
                    if c == env.copy_button {
                        selector.copy(core);
                        selector.deactivate();
                    } else if c == env.select_button {
                        if !selector.is_active() {
                            selector.activate()
                        }
                    } else if c == env.paste_button {
                        selector.paste(core);
                    } else if c == env.quit_buton {
                        return NextAction::Break;
                    }
                }
                _ => (),
            },
        }
        NextAction::Continue
    }
}
