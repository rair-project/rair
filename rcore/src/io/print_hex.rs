/*
 * print_hex: commands handling hex printing.
 * Copyright (C) 2019  Oddcoder
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use core::*;
use helper::*;
use std::cmp;
use std::io::Write;
use writer::*;
use yansi::Paint;

#[derive(Default)]
pub struct PrintHex {}

impl PrintHex {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for PrintHex {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        // we can always optimize by try and using pread an vread.
        // If they fail, only then we might want to attempt the sparce version.
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return;
        }
        let size = match str_to_num(&args[0]) {
            Ok(s) => s,
            Err(e) => {
                return error_msg(
                    core,
                    &e.to_string(),
                    &format!("Expect Hex, binary, Octal or Decimal value but found {} instead", Paint::default(&args[0]).italic()),
                )
            }
        };
        if size == 0 {
            return;
        }
        let loc = core.get_loc();
        let data_or_no_data = match core.mode {
            AddrMode::Phy => core.io.pread_sparce(loc, size),
            AddrMode::Vir => core.io.vread_sparce(loc, size),
        };
        let data = match data_or_no_data {
            Ok(d) => d,
            Err(e) => return error_msg(core, "Read Failed", &e.to_string()),
        };
        let banner = core.color_palette[5];
        let na = core.color_palette[4];
        writeln!(
            core.stdout,
            "{}",
            Paint::rgb(banner.0, banner.1, banner.2, "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF")
        )
        .unwrap();
        for i in (0..size).step_by(16) {
            write!(core.stdout, "{} ", Paint::rgb(banner.0, banner.1, banner.2, format!("0x{:08x}", loc + i))).unwrap();
            let mut ascii = Writer::new_buf();
            let mut hex = Writer::new_buf();
            for j in i..cmp::min(i + 16, size) {
                if let Some(c) = data.get(&(j + loc)) {
                    if j % 2 == 0 {
                        write!(hex, "{:02x}", c).unwrap();
                    } else {
                        write!(hex, "{:02x} ", c).unwrap();
                    }
                    if *c >= 0x21 && *c <= 0x7E {
                        write!(ascii, "{}", *c as char).unwrap()
                    } else {
                        write!(ascii, "{}", Paint::rgb(na.0, na.1, na.2, ".")).unwrap();
                    }
                } else {
                    if j % 2 == 0 {
                        write!(hex, "##").unwrap();
                    } else {
                        write!(hex, "## ").unwrap();
                    }
                    write!(ascii, "{}", Paint::rgb(na.0, na.1, na.2, "#")).unwrap();
                }
            }
            writeln!(core.stdout, "{: <40} {}", hex.utf8_string().unwrap(), ascii.utf8_string().unwrap()).unwrap();
        }
    }
    fn help(&self, core: &mut Core) {
        help(core, &"printHex", &"px", vec![("[size]", "View data of at current location in hex format.")]);
    }
}

#[cfg(test)]
mod test_print_hex {
    use super::*;
    use rio::*;
    use std::path::Path;
    use test_file::*;
    use writer::Writer;
    use yansi::Paint;

    #[test]
    fn test_help() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let px = PrintHex::new();
        px.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [printHex | px]\n\nUsage:\npx [size]\tView data of at current location in hex format.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    fn test_px_cb(path: &Path) {
        Paint::disable();
        let mut core = Core::new();
        let mut px = PrintHex::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        px.run(&mut core, &["0x0".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 00                                       .\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x2".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001                                     ..\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x3".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 01                                  ...\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x4".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102                                ....\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x5".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 03                             .....\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x6".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305                           ......\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x7".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 08                        .......\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x8".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d                      ........\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x9".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 15                   .........\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xa".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522                 .........\"\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xb".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 37              .........\"7\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xc".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759            .........\"7Y\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xd".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90         .........\"7Y.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xe".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9       .........\"7Y..\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0xf".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 79    .........\"7Y..y\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x10".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x11".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db                                       .\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x12".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d                                     .=\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x13".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 18                                  .=.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x14".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855                                .=.U\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x15".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6d                             .=.Um\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x16".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2                           .=.Um.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x17".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2f                        .=.Um./\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x18".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1                      .=.Um./.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x19".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 20                   .=.Um./..\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1a".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011                 .=.Um./...\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1b".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 31              .=.Um./...1\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1c".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142            .=.Um./...1B\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1d".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73         .=.Um./...1Bs\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1e".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5       .=.Um./...1Bs.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x1f".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28    .=.Um./...1Bs.(\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x20".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x21".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n\
             0x00000020 05                                       .\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x22".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n\
             0x00000020 05e2                                     ..\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        px.run(&mut core, &["0x23".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000010 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n\
             0x00000020 05e2 e7                                  ...\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_px() {
        operate_on_file(&test_px_cb, DATA);
    }

    fn test_px_vir_cb(path: &Path) {
        Paint::disable();
        let mut core = Core::new();
        let mut px = PrintHex::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        px.run(&mut core, &["0x0".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.mode = AddrMode::Vir;
        core.io.map(0x0, 0x500, 0x10).unwrap();
        core.io.map(0x15, 0x515, 0x5).unwrap();
        core.io.map(0x10, 0x520, 0x20).unwrap();
        core.io.map(0x20, 0x540, 0x20).unwrap();
        core.set_loc(0x500);
        px.run(&mut core, &["0x65".to_string()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
             0x00000500 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
             0x00000510 #### #### ##c2 2ff1 2011 #### #### ####  #####./...######\n\
             0x00000520 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n\
             0x00000530 05e2 e7c9 b079 29a2 cb6d 38a5 dd82 5fe1  .....y)..m8..._.\n\
             0x00000540 05e2 e7c9 b079 29a2 cb6d 38a5 dd82 5fe1  .....y)..m8..._.\n\
             0x00000550 4021 6182 e365 48ad f5a2 9739 d009 d9e2  @!a..eH....9....\n\
             0x00000560 #### #### ##                             #####\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_px_vir() {
        operate_on_file(&test_px_vir_cb, DATA);
    }
}
