use super::helper::one_byte;
use rair_core::{Core, HexEnv, HexWithoutEnv};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

#[derive(Default)]
pub struct VisualHexEnv {
    pub non_visual: HexEnv,
    pub edit_mode_button: char,
    pub select_mode_button: char,
    pub command_mode_button: char,
    pub quit_buton: char,
    pub copy_button: char,
    pub select_button: char,
    pub paste_button: char,
    pub show_help: bool,
}

impl VisualHexEnv {
    pub(super) fn new(core: &mut Core) -> Self {
        let _ = HexWithoutEnv::new(core);
        let non_visual = HexEnv::default();
        let env_locked = core.env.clone();
        let mut env = env_locked.write();
        if !env.contains("hex.editButton") {
            env.add_str_with_cb(
                "hex.editButton",
                "i",
                "Button used to toggle edit mode",
                core,
                one_byte,
            )
            .unwrap();
        }
        if !env.contains("hex.selectButton") {
            env.add_str_with_cb(
                "hex.selectButton",
                "v",
                "Button used to toggle visual mode",
                core,
                one_byte,
            )
            .unwrap();
        }
        if !env.contains("hex.commandButton") {
            env.add_str_with_cb(
                "hex.commandButton",
                ":",
                "Button used to toggle command mode",
                core,
                one_byte,
            )
            .unwrap();
        }
        if !env.contains("hex.quitButton") {
            env.add_str_with_cb(
                "hex.quitButton",
                "q",
                "Button used to return back to interactive prompt",
                core,
                one_byte,
            )
            .unwrap();
        }
        if !env.contains("hex.copyButton") {
            env.add_str_with_cb(
                "hex.copyButton",
                "y",
                "Button used to copy in select mode",
                core,
                one_byte,
            )
            .unwrap();
        }
        if !env.contains("hex.SelectionStartButton") {
            env.add_str_with_cb(
                "hex.SelectionStartButton",
                "v",
                "Button used to start selection in select mode",
                core,
                one_byte,
            )
            .unwrap();
        }
        if !env.contains("hex.pasteButton") {
            env.add_str_with_cb(
                "hex.pasteButton",
                "p",
                "Button used to paste in select mode",
                core,
                one_byte,
            )
            .unwrap();
        }
        if !env.contains("hex.showHelp") {
            env.add_bool("hex.showHelp", true, "Show help in visual modes")
                .unwrap();
        }
        Self {
            non_visual,
            ..Default::default()
        }
    }

    pub(super) fn get_env(&mut self, core: &mut Core) -> &Self {
        self.non_visual.get_env(core);
        let env = core.env.read();
        self.edit_mode_button = env
            .get_str("hex.editButton")
            .unwrap()
            .chars()
            .next()
            .unwrap();
        self.select_mode_button = env
            .get_str("hex.selectButton")
            .unwrap()
            .chars()
            .next()
            .unwrap();
        self.command_mode_button = env
            .get_str("hex.commandButton")
            .unwrap()
            .chars()
            .next()
            .unwrap();
        self.quit_buton = env
            .get_str("hex.quitButton")
            .unwrap()
            .chars()
            .next()
            .unwrap();
        self.copy_button = env
            .get_str("hex.copyButton")
            .unwrap()
            .chars()
            .next()
            .unwrap();
        self.select_button = env
            .get_str("hex.SelectionStartButton")
            .unwrap()
            .chars()
            .next()
            .unwrap();
        self.paste_button = env
            .get_str("hex.pasteButton")
            .unwrap()
            .chars()
            .next()
            .unwrap();
        self.show_help = env.get_bool("hex.showHelp").unwrap();
        self
    }
    pub fn render_addresses_banner(&self, core: &Core) -> Line {
        let header = format!("{}", core.mode);
        let (r, g, b) = self.non_visual.banner;
        Span::styled(header, Style::default().fg(Color::Rgb(r, g, b))).into()
    }
    pub fn render_addresses(&self, loc: u64) -> Line {
        let addr = format!("0x{:08x} ", loc);
        let (r, g, b) = self.non_visual.banner;
        Span::styled(addr, Style::default().fg(Color::Rgb(r, g, b))).into()
    }
    pub fn render_ascii_banner(&self) -> Line {
        let (r, g, b) = self.non_visual.banner;
        Span::styled("0123456789ABCDEF", Style::default().fg(Color::Rgb(r, g, b))).into()
    }
    pub fn render_hex_banner(&self) -> Line {
        let (r, g, b) = self.non_visual.banner;
        Span::styled(
            " 0 1  2 3  4 5  6 7  8 9  A B  C D  E F",
            Style::default().fg(Color::Rgb(r, g, b)),
        )
        .into()
    }

    pub fn render_ascii(&self, byte: Option<u8>) -> Span {
        let Some(c) = byte else {
            return Span::from(self.non_visual.gap.to_string());
        };
        if (0x21..=0x7E).contains(&c) {
            Span::from(format!("{}", c as char))
        } else {
            let (r, g, b) = self.non_visual.na;
            Span::styled(
                self.non_visual.noprint.to_string(),
                Style::default().fg(Color::Rgb(r, g, b)),
            )
        }
    }
}
