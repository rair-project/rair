use cli_clipboard::{ClipboardContext, ClipboardProvider};
use rair_core::Core;
use ratatui::{
    style::{Style, Stylize},
    text::Span,
};

pub enum Selector {
    Inactive { loc: u64 },
    Active { start: u64, end: u64 },
    ReverseActive { start: u64, end: u64 },
}

impl Selector {
    pub fn new(core: &Core) -> Self {
        Self::Inactive {
            loc: core.get_loc(),
        }
    }
    pub fn is_active(&self) -> bool {
        match self {
            Selector::Inactive { .. } => false,
            Selector::Active { .. } | Selector::ReverseActive { .. } => true,
        }
    }
    pub fn activate(&mut self) {
        if let Self::Inactive { loc } = self {
            *self = Self::Active {
                start: *loc,
                end: *loc,
            }
        } else {
            unreachable!()
        }
    }
    pub fn deactivate(&mut self) {
        match self {
            Selector::Inactive { .. } => (),
            Selector::Active { end, .. } => *self = Self::Inactive { loc: *end },
            Selector::ReverseActive { start, .. } => *self = Self::Inactive { loc: *start },
        }
    }
    pub fn span_hex<'a>(&self, hex: String, loc: u64) -> Span<'a> {
        let loc2 = loc;
        match self {
            Selector::Inactive { loc } => {
                if loc2 == *loc {
                    Span::styled(hex, Style::default().reversed())
                } else {
                    Span::from(hex)
                }
            }
            Selector::Active { start, end } | Selector::ReverseActive { start, end } => {
                if loc2 >= *start && loc2 <= *end {
                    Span::styled(hex, Style::default().reversed())
                } else {
                    Span::from(hex)
                }
            }
        }
    }
    pub fn span_ascii<'a>(&self, ascii: Span<'a>, loc: u64) -> Span<'a> {
        let loc2 = loc;
        match self {
            Selector::Inactive { loc } => {
                if loc2 == *loc {
                    ascii.patch_style(Style::default().reversed())
                } else {
                    ascii
                }
            }
            Selector::Active { start, end } | Selector::ReverseActive { start, end } => {
                if loc2 >= *start && loc2 <= *end {
                    ascii.patch_style(Style::default().reversed())
                } else {
                    ascii
                }
            }
        }
    }
    pub fn right(&mut self, core: &mut Core, max_bytes: u64) {
        match self {
            Selector::Inactive { loc } => {
                if *loc == u64::MAX {
                    return;
                }
                *loc += 1;
                let loc2 = core.get_loc();
                if *loc >= loc2 + max_bytes {
                    core.set_loc(loc2 + 16);
                }
            }
            Selector::Active { end, .. } => {
                if *end == u64::MAX {
                    return;
                }
                *end += 1;
                let loc2 = core.get_loc();
                if *end >= loc2 + max_bytes {
                    core.set_loc(loc2 + 16);
                }
            }
            Selector::ReverseActive { start, end } => {
                if *start == u64::MAX {
                    return;
                }
                *start += 1;
                let loc2 = core.get_loc();
                if *start >= loc2 + max_bytes {
                    core.set_loc(loc2 + 16);
                }
                if *start > *end {
                    *self = Selector::Active {
                        start: *end,
                        end: *start,
                    }
                }
            }
        }
    }
    pub fn left(&mut self, core: &mut Core) {
        match self {
            Selector::Inactive { loc } => {
                if *loc == u64::MIN {
                    return;
                }
                *loc -= 1;
                let loc2 = core.get_loc();
                if *loc < loc2 {
                    core.set_loc(loc2 - 16);
                }
            }
            Selector::Active { start, end } => {
                if *end == u64::MIN {
                    return;
                }
                *end -= 1;
                let loc2: u64 = core.get_loc();
                if *end < loc2 {
                    core.set_loc(loc2 - 16);
                }
                if *start > *end {
                    *self = Selector::ReverseActive {
                        start: *end,
                        end: *start,
                    }
                }
            }
            Selector::ReverseActive { start, .. } => {
                if *start == u64::MIN {
                    return;
                }
                *start -= 1;
                let loc2: u64 = core.get_loc();
                if *start < loc2 {
                    core.set_loc(loc2 - 16);
                }
            }
        }
    }
    pub fn up(&mut self, core: &mut Core) {
        match self {
            Selector::Inactive { loc } => {
                if loc.wrapping_sub(16) > *loc {
                    return;
                }
                *loc -= 16;
                let loc2 = core.get_loc();
                if *loc < loc2 {
                    core.set_loc(loc2 - 16);
                }
            }
            Selector::Active { start, end } => {
                if end.wrapping_sub(16) > *end {
                    return;
                }
                *end -= 16;
                let loc2: u64 = core.get_loc();
                if *end < loc2 {
                    core.set_loc(loc2 - 16);
                }
                if *start > *end {
                    *self = Selector::ReverseActive {
                        start: *end,
                        end: *start,
                    }
                }
            }
            Selector::ReverseActive { start, .. } => {
                if start.wrapping_sub(16) > *start {
                    return;
                }
                *start -= 16;
                let loc2: u64 = core.get_loc();
                if *start < loc2 {
                    core.set_loc(loc2 - 16);
                }
            }
        }
    }
    pub fn down(&mut self, core: &mut Core, max_bytes: u64) {
        match self {
            Selector::Inactive { loc } => {
                if loc.wrapping_add(16) < *loc {
                    return;
                }
                *loc += 16;
                let loc2 = core.get_loc();
                if *loc >= loc2 + max_bytes {
                    core.set_loc(loc2 + 16);
                }
            }
            Selector::Active { end, .. } => {
                if end.wrapping_add(16) < *end {
                    return;
                }
                *end += 16;
                let loc2 = core.get_loc();
                if *end >= loc2 + max_bytes {
                    core.set_loc(loc2 + 16);
                }
            }
            Selector::ReverseActive { start, end } => {
                if start.wrapping_add(16) < *start {
                    return;
                }
                *start += 16;
                let loc2 = core.get_loc();
                if *start >= loc2 + max_bytes {
                    core.set_loc(loc2 + 16);
                }
                if *start > *end {
                    *self = Selector::Active {
                        start: *end,
                        end: *start,
                    }
                }
            }
        }
    }

    pub fn copy(&mut self, core: &mut Core) {
        match self {
            Selector::Inactive { .. } => (),
            Selector::Active { start, end } | Selector::ReverseActive { start, end } => {
                let mut buf = vec![0u8; (*end - *start + 1) as usize];
                if core.read(*start, &mut buf).is_err() {
                    return;
                }
                let data = bytes_to_hex(&buf);
                let mut ctx = ClipboardContext::new().unwrap();
                ctx.set_contents(data).unwrap();
            }
        }
    }

    pub fn paste(&mut self, core: &mut Core) {
        let Selector::Inactive { loc } = self else {
            return;
        };
        let mut ctx = ClipboardContext::new().unwrap();
        let data = ctx.get_contents().unwrap();
        let Some(buf) = hex_to_bytes(&data) else {
            return;
        };
        let _ = core.write(*loc, &buf);
        // todo fast forward cursor after paste
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex_string = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex_string.push_str(&format!("{:02x}", byte));
    }

    hex_string
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        // Get the two-character substring
        let byte_str = &hex[i..i + 2];
        // Parse the substring as a byte
        match u8::from_str_radix(byte_str, 16) {
            Ok(byte) => bytes.push(byte),
            Err(_) => return None,
        }
    }
    Some(bytes)
}
