use ratatui::{
    layout::{Position, Rect},
    text::Line,
    Frame,
};
use std::cmp::max;

#[derive(Default)]
pub struct Command {
    input: String,
    cursor: usize,
}

impl Command {
    pub fn move_cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }
    pub fn move_cursor_right(&mut self) {
        self.cursor = max(self.cursor.saturating_add(1), self.input.len())
    }
    pub fn delete_char(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let from_left_to_custor = self.cursor - 1;
        // Getting all characters before the selected character.
        let before_char_to_delete = self.input.chars().take(from_left_to_custor);
        // Getting all characters after selected character.
        let after_char_to_delete = self.input.chars().skip(self.cursor);
        self.input = before_char_to_delete.chain(after_char_to_delete).collect();
        self.move_cursor_left();
    }
    pub fn enter_char(&mut self, c: char) {
        let index = self
            .input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor)
            .unwrap_or(self.input.len());
        self.input.insert(index, c);
        self.move_cursor_right();
    }
    pub fn get_str(&self) -> &str {
        &self.input
    }
    pub fn render(&self, f: &mut Frame, chunk: Rect) {
        let input = Line::from(format!(":{}", self.input));
        f.render_widget(input, chunk);
        f.set_cursor_position(Position::new(chunk.x + self.cursor as u16 + 1, chunk.y));
    }
}
