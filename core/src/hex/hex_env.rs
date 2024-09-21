use super::helper::one_byte;
use crate::{is_color, Core, Writer};
use std::io::Write;
use yansi::Paint;

#[derive(Clone)]
pub struct HexEnv {
    // color for banner and offsets
    pub banner: (u8, u8, u8),
    // color for the ascii part that
    // is not printable
    pub na: (u8, u8, u8),
    // character to print in case of gap
    pub gap: char,
    // character to print (with na color)
    pub noprint: char,
}

impl HexEnv {
    pub(super) fn new(core: &mut Core) -> Self {
        let env = core.env.clone();
        env.write()
            .add_str_with_cb(
                "hex.headerColor",
                "color.6",
                "Color used in the header of when using commands working with hex data",
                core,
                is_color,
            )
            .unwrap();
        env.write()
        .add_str_with_cb(
            "hex.nonPrintColor",
            "color.5",
            "Color used in the Ascii section for non printable ASCII when using commands that work with hex data",
            core,
            is_color,
        )
        .unwrap();
        env.write()
        .add_str_with_cb(
            "hex.nonPrintReplace",
            ".",
            "Text used in the Ascii section to replace non printable ASCII when using when using commands that work with hex data",
            core,
            one_byte,
        )
        .unwrap();
        env.write()
            .add_str_with_cb(
                "hex.gapReplace",
                "#",
                "Text used to replace gaps when using when using commands that work with hex data",
                core,
                one_byte,
            )
            .unwrap();

        Self {
            banner: (0, 0, 0),
            na: (0, 0, 0),
            gap: char::default(),
            noprint: char::default(),
        }
    }
    pub(super) fn get_env(&mut self, core: &mut Core) -> &Self {
        let env = core.env.read();
        let color = env.get_str("hex.headerColor").unwrap();
        self.banner = env.get_color(color).unwrap();
        let color = env.get_str("hex.nonPrintColor").unwrap();
        self.na = core.env.read().get_color(color).unwrap();
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
    pub fn print_addr(&self, writer: &mut Writer, loc: u64) {
        let loc = format!("0x{loc:08x}");
        let (r, g, b) = self.banner;
        let loc_colored = loc.rgb(r, g, b);
        write!(writer, "{loc_colored} ").unwrap();
    }
    // print hex data all while taking care of extra white space
    pub fn print_hex(&self, data: Option<u8>, writer: &mut Writer, space_after: bool) {
        let space = if space_after { " " } else { "" };
        if let Some(c) = data {
            write!(writer, "{c:02x}{space}").unwrap();
        } else {
            write!(writer, "{}{}{space}", self.gap, self.gap).unwrap();
        }
    }
    // print ascii data while taking care of non printable characters and coloring
    pub fn print_ascii(&self, data: Option<u8>, writer: &mut Writer) {
        let (r, g, b) = self.na;
        if let Some(c) = data {
            if (0x21..=0x7E).contains(&c) {
                write!(writer, "{}", c as char).unwrap();
            } else {
                write!(writer, "{}", self.noprint.rgb(r, g, b)).unwrap();
            }
        } else {
            write!(writer, "{}", self.gap.rgb(r, g, b)).unwrap();
        }
    }
}
