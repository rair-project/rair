//! Red black tree color operations.

#[derive(PartialEq)]
pub enum Color {
    Red,
    Black,
}

impl Color {
    pub fn flip(&mut self) {
        match self {
            Color::Red => *self = Color::Black,
            Color::Black => *self = Color::Red,
        }
    }
}
