use super::helper::one_byte;
use crate::{is_color, Core, Writer};
use std::io::Write;
use yansi::Paint;

#[derive(Clone, Default)]
pub struct HexEnv {
    // color for banner and offsets
    pub banner: (u8, u8, u8),
    // color for the ascii part that
    // is not printable
    pub na: (u8, u8, u8),
    // color for highlighting
    pub highlight: (u8, u8, u8),
    // character to print in case of gap
    pub gap: char,
    // character to print (with na color)
    pub noprint: char,
    // separator between side by side views
    pub separator: String,
}

impl HexEnv {
    pub(super) fn new(core: &mut Core) -> Self {
        let env_lock = core.env.clone();
        let mut env = env_lock.write();
        if !env.contains("hex.headerColor") {
            env.add_str_with_cb(
                "hex.headerColor",
                "color.6",
                "Color used in the header of when using commands working with hex data",
                core,
                is_color,
            )
            .unwrap();
        }
        if !env.contains("hex.nonPrintColor") {
            env.add_str_with_cb(
            "hex.nonPrintColor",
            "color.5",
            "Color used in the Ascii section for non printable ASCII when using commands that work with hex data",
            core,
            is_color,
        ).unwrap();
        }
        if !env.contains("hex.nonPrintReplace") {
            env.add_str_with_cb(
            "hex.nonPrintReplace",
            ".",
            "Text used in the Ascii section to replace non printable ASCII when using when using commands that work with hex data",
            core,
            one_byte,
        ).unwrap();
        }
        if !env.contains("hex.gapReplace") {
            env.add_str_with_cb(
                "hex.gapReplace",
                "#",
                "Text used to replace gaps when using when using commands that work with hex data",
                core,
                one_byte,
            )
            .unwrap();
        }
        if !env.contains("hex.separator") {
            env.add_str(
                "hex.separator",
                "││",
                "Separator between the two side by side views",
            )
            .unwrap();
        }
        if !env.contains("hex.highlight") {
            env.add_str_with_cb(
                "hex.highlight",
                "color.1",
                "Color used to highlight different sections when needed",
                core,
                is_color,
            )
            .unwrap();
        }
        Self {
            banner: (0, 0, 0),
            na: (0, 0, 0),
            highlight: (0, 0, 0),
            gap: char::default(),
            noprint: char::default(),
            separator: String::new(),
        }
    }
    pub fn get_env(&mut self, core: &mut Core) -> &Self {
        let env = core.env.read();
        let color = env.get_str("hex.headerColor").unwrap();
        self.banner = env.get_color(color).unwrap();
        let color = env.get_str("hex.nonPrintColor").unwrap();
        self.na = core.env.read().get_color(color).unwrap();
        let color = env.get_str("hex.highlight").unwrap();
        self.highlight = core.env.read().get_color(color).unwrap();
        self.gap = env
            .get_str("hex.gapReplace")
            .unwrap()
            .chars()
            .next()
            .unwrap();
        self.noprint = env
            .get_str("hex.nonPrintReplace")
            .unwrap()
            .chars()
            .next()
            .unwrap();
        env.get_str("hex.separator")
            .unwrap()
            .clone_into(&mut self.separator);
        self
    }
    pub fn print_banner_with_newline(&self, writer: &mut Writer, newline: bool) {
        let nl = if newline { "\n" } else { "" };
        write!(
            writer,
            "{}{nl}",
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF".rgb(
                self.banner.0,
                self.banner.1,
                self.banner.2,
            )
        )
        .unwrap();
    }
    pub fn print_banner(&self, writer: &mut Writer) {
        self.print_banner_with_newline(writer, true);
    }
    pub fn print_double_banner(&self, writer: &mut Writer) {
        self.print_banner_with_newline(writer, false);
        self.print_separator(writer);
        self.print_banner_with_newline(writer, true);
    }
    pub fn print_addr(&self, writer: &mut Writer, loc: u64) {
        let loc = format!("0x{loc:08x}");
        let (r, g, b) = self.banner;
        let loc_colored = loc.rgb(r, g, b);
        write!(writer, "{loc_colored} ").unwrap();
    }
    pub fn print_hex_with_highlight(
        &self,
        data: Option<u8>,
        writer: &mut Writer,
        space_after: bool,
        highlight: bool,
    ) {
        let space = if space_after { " " } else { "" };
        let hex: String = if let Some(c) = data {
            format!("{c:02x}")
        } else {
            format!("{}{}", self.gap, self.gap)
        };
        if highlight {
            let (r, g, b) = self.highlight;
            write!(writer, "{}{space}", hex.on_rgb(r, g, b)).unwrap();
        } else {
            write!(writer, "{hex}{space}").unwrap();
        }
    }

    // print hex data all while taking care of extra white space
    pub fn print_hex(&self, data: Option<u8>, writer: &mut Writer, space_after: bool) {
        self.print_hex_with_highlight(data, writer, space_after, false);
    }
    pub fn print_ascii_with_highlight(
        &self,
        data: Option<u8>,
        writer: &mut Writer,
        highlight: bool,
    ) {
        let (r, g, b) = self.na;
        let ascii = if let Some(c) = data {
            if (0x21..=0x7E).contains(&c) {
                format!("{}", c as char)
            } else {
                format!("{}", self.noprint.rgb(r, g, b))
            }
        } else {
            format!("{}", self.gap.rgb(r, g, b))
        };
        if highlight {
            let (r, g, b) = self.highlight;
            write!(writer, "{}", ascii.on_rgb(r, g, b)).unwrap();
        } else {
            write!(writer, "{ascii}").unwrap();
        }
    }

    // print ascii data while taking care of non printable characters and coloring
    pub fn print_ascii(&self, data: Option<u8>, writer: &mut Writer) {
        self.print_ascii_with_highlight(data, writer, false);
    }
    pub fn print_separator(&self, writer: &mut Writer) {
        let (r, g, b) = self.banner;
        write!(writer, "    {}    ", self.separator.rgb(r, g, b)).unwrap();
    }
}
