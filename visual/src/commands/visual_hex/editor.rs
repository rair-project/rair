use rair_core::Core;
use ratatui::{
    style::{Style, Stylize},
    text::Span,
};

enum EditorMode {
    Hex { first_char: bool },
    Ascii,
}
pub struct Editor {
    loc: u64,
    mode: EditorMode,
}

impl Editor {
    pub fn new(core: &Core) -> Self {
        Editor {
            loc: core.get_loc(),
            mode: EditorMode::Hex { first_char: true },
        }
    }
    pub fn switch_mode(&mut self) {
        match self.mode {
            EditorMode::Hex { .. } => self.mode = EditorMode::Ascii,
            EditorMode::Ascii => self.mode = EditorMode::Hex { first_char: true },
        }
    }
    /// invariant hex must be exactly two characters!
    pub fn span_hex<'a>(&self, hex: String, loc: u64) -> Vec<Span<'a>> {
        if hex.len() != 2 {
            panic!("Invariant broken: hex must be exactly two characters")
        }
        if loc != self.loc {
            return vec![Span::from(hex)];
        }
        match self.mode {
            EditorMode::Ascii => vec![Span::styled(hex, Style::default().bold())],
            EditorMode::Hex { first_char } => {
                let chars: Vec<_> = hex.chars().collect();
                if first_char {
                    vec![
                        Span::styled(String::from(chars[0]), Style::default().reversed()),
                        Span::from(String::from(chars[1])),
                    ]
                } else {
                    vec![
                        Span::from(String::from(chars[0])),
                        Span::styled(String::from(chars[1]), Style::default().reversed()),
                    ]
                }
            }
        }
    }

    pub fn span_ascii<'a>(&self, ascii: Span<'a>, loc: u64) -> Span<'a> {
        if loc != self.loc {
            return ascii;
        }
        match self.mode {
            EditorMode::Ascii => ascii.patch_style(Style::default().reversed()),
            EditorMode::Hex { .. } => ascii.patch_style(Style::default().bold()),
        }
    }
    pub fn left(&mut self, core: &mut Core) {
        match self.mode {
            EditorMode::Hex { first_char: false } => {
                self.mode = EditorMode::Hex { first_char: true };
                return;
            }
            EditorMode::Hex { first_char: true } => {
                if self.loc == u64::MIN {
                    return;
                }
                self.mode = EditorMode::Hex { first_char: false };
            }
            EditorMode::Ascii => {
                if self.loc == u64::MIN {
                    return;
                }
            }
        }
        self.loc -= 1;
        let loc = core.get_loc();
        if self.loc < loc {
            core.set_loc(loc - 16);
        }
    }
    pub fn right(&mut self, core: &mut Core, max_bytes: u64) {
        match self.mode {
            EditorMode::Hex { first_char: true } => {
                self.mode = EditorMode::Hex { first_char: false };
                return;
            }
            EditorMode::Hex { first_char: false } => {
                if self.loc == u64::MAX {
                    return;
                }
                self.mode = EditorMode::Hex { first_char: true };
            }
            EditorMode::Ascii => {
                if self.loc == u64::MAX {
                    return;
                }
            }
        }
        self.loc += 1;
        let loc = core.get_loc();
        if self.loc >= loc + max_bytes {
            core.set_loc(loc + 16);
        }
    }
    pub fn down(&mut self, core: &mut Core, max_bytes: u64) {
        if self.loc.wrapping_add(16) < self.loc {
            return;
        }
        self.loc += 16;
        let loc = core.get_loc();
        if self.loc >= loc + max_bytes {
            core.set_loc(loc + 16);
        }
    }
    pub fn up(&mut self, core: &mut Core) {
        if self.loc.wrapping_sub(16) > self.loc {
            return;
        }
        self.loc -= 16;
        let loc = core.get_loc();
        if self.loc < loc {
            core.set_loc(loc - 16);
        }
    }
    pub fn write_char_ascii(&self, core: &mut Core, c: char) {
        let byte = [c as u8];
        drop(core.write(self.loc, &byte));
    }
    fn char_to_hex(c: char) -> Option<u8> {
        match c {
            '0' => Some(0x0),
            '1' => Some(0x1),
            '2' => Some(0x2),
            '3' => Some(0x3),
            '4' => Some(0x4),
            '5' => Some(0x5),
            '6' => Some(0x6),
            '7' => Some(0x7),
            '8' => Some(0x8),
            '9' => Some(0x9),
            'a' => Some(0xa),
            'b' => Some(0xb),
            'c' => Some(0xc),
            'd' => Some(0xd),
            'e' => Some(0xe),
            'f' => Some(0xf),
            'A' => Some(0xA),
            'B' => Some(0xB),
            'C' => Some(0xC),
            'D' => Some(0xD),
            'E' => Some(0xE),
            'F' => Some(0xF),
            _ => None,
        }
    }
    pub fn write_char_hex(&self, core: &mut Core, c: char, mask: u8, shift: u8) {
        let Some(c) = Self::char_to_hex(c) else {
            return;
        };
        let mut byte = [0];
        let _ = core.read(self.loc, &mut byte);
        byte[0] = (byte[0] & mask) | (c << shift);
        let _ = core.write(self.loc, &byte);
    }
    pub fn write_char(&self, core: &mut Core, c: char) {
        match self.mode {
            EditorMode::Hex { first_char: true } => self.write_char_hex(core, c, 0xf, 4),
            EditorMode::Hex { first_char: false } => self.write_char_hex(core, c, 0xf0, 0),
            EditorMode::Ascii => self.write_char_ascii(core, c),
        }
    }
}
