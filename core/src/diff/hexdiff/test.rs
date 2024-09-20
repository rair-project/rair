use std::path::Path;

use rair_io::IoMode;
use test_file::{operate_on_file, DATA};

use crate::{diff::hexdiff::HexDiff, CmdOps, Core, Writer};

#[test]
fn test_help() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let hd = HexDiff::new(&mut core);
    hd.help(&mut core);
    assert_eq!(
        core.stdout.utf8_string().unwrap(),
        "Commands: [hexDiff | hd]\n\
        Usage:\n\
        hd [addr] [size]\t\tPrint binary diff between current location and [addr] for [size] bytes.\n\
        hd [addr1] [addr2] [size]\tPrint binary diff between [addr1] and [addr2] for [size] bytes.\n"
    );
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
}

#[test]
fn test_hd_0_args() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run("hd", &[]);
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected between 2 and 3 arguments, found 0.\n"
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
    }
    operate_on_file(&test_hd_cb, DATA);
}

#[test]
fn test_hd_1_args() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run("hd", &["0x5".to_owned()]);
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected between 2 and 3 arguments, found 1.\n"
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
    }
    operate_on_file(&test_hd_cb, DATA);
}

#[test]
fn test_hd_2_args() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run("hd", &["0x5".to_owned(), "0x25".to_owned()]);
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
 0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb    ││    0x00000005 0508 0d15 2237 5990 e979 62db 3d18 556d  ....\"7Y..yb.=.Um\n\
 0x00000000 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.    ││    0x00000005 c22f f120 1131 4273 b528 dd05 e2e7 c9b0  ./...1Bs.(......\n\
 0x00000000 05e2 e7c9 b0                             .....               ││    0x00000005 7929 a2cb 6d                             y)..m\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}

#[test]
fn test_hd_4_args() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &[
                "0x5".to_owned(),
                "0x5".to_owned(),
                "0x5".to_owned(),
                "0x5".to_owned(),
            ],
        );
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected between 2 and 3 arguments, found 4.\n"
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_2_args_1_bad() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run("hd", &["alex".to_owned(), "0x25".to_owned()]);
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse addr\ninvalid digit found in string\n"
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_2_args_2_bad() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run("hd", &["0x5".to_owned(), "alex".to_owned()]);
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse size\ninvalid digit found in string\n"
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
    }
    operate_on_file(&test_hd_cb, DATA);
}

#[test]
fn test_hd_3_args_1_bad() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["alex".to_owned(), "0x25".to_owned(), "0x25".to_owned()],
        );
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse addr1\ninvalid digit found in string\n"
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_3_args_2_bad() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x5".to_owned(), "alex".to_owned(), "0x25".to_owned()],
        );
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse addr2\ninvalid digit found in string\n"
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
    }
    operate_on_file(&test_hd_cb, DATA);
}

#[test]
fn test_hd_3_args_3_bad() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x5".to_owned(), "0x25".to_owned(), "alex".to_owned()],
        );
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to parse size\ninvalid digit found in string\n"
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
    }
    operate_on_file(&test_hd_cb, DATA);
}

#[test]
fn test_hd_0() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x0".to_owned()],
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_1() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x1".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 00                                       .                   ││    0x00000000 00                                       .\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_2() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x2".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001                                     ..                  ││    0x00000000 0001                                     ..\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_3() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x3".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 01                                  ...                 ││    0x00000000 0001 01                                  ...\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_4() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x4".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 0102                                ....                ││    0x00000000 0001 0102                                ....\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_5() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x5".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 0102 03                             .....               ││    0x00000000 0001 0102 03                             .....\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_6() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x6".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 0102 0305                           ......              ││    0x00000000 0001 0102 0305                           ......\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_7() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x7".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 0102 0305 08                        .......             ││    0x00000000 0001 0102 0305 08                        .......\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_8() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x8".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 0102 0305 080d                      ........            ││    0x00000000 0001 0102 0305 080d                      ........\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_9() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x9".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 0102 0305 080d 15                   .........           ││    0x00000000 0001 0102 0305 080d 15                   .........\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_a() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0xa".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 0102 0305 080d 1522                 .........\"          ││    0x00000000 0001 0102 0305 080d 1522                 .........\"\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_b() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0xb".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 0102 0305 080d 1522 37              .........\"7         ││    0x00000000 0001 0102 0305 080d 1522 37              .........\"7\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_c() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0xc".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 0102 0305 080d 1522 3759            .........\"7Y        ││    0x00000000 0001 0102 0305 080d 1522 3759            .........\"7Y\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_d() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0xd".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 0102 0305 080d 1522 3759 90         .........\"7Y.       ││    0x00000000 0001 0102 0305 080d 1522 3759 90         .........\"7Y.\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_e() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0xe".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
0x00000000 0001 0102 0305 080d 1522 3759 90e9       .........\"7Y..      ││    0x00000000 0001 0102 0305 080d 1522 3759 90e9       .........\"7Y..\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
#[test]
fn test_hd_f() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0xf".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
 0x00000000 0001 0102 0305 080d 1522 3759 90e9 79    .........\"7Y..y     ││    0x00000000 0001 0102 0305 080d 1522 3759 90e9 79    .........\"7Y..y\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}

#[test]
fn test_hd_10() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x10".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
 0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb    ││    0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}

#[test]
fn test_hd_11() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "0x11".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
 0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb    ││    0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
 0x00000000 db                                       .                   ││    0x00000000 db                                       .\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}

#[test]
fn test_hd_100() {
    fn test_hd_cb(path: &Path) {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        core.run(
            "hd",
            &["0x0".to_owned(), "0x0".to_owned(), "100".to_owned()],
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(),
"- offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF    ││    - offset -  0 1  2 3  4 5  6 7  8 9  A B  C D  E F  0123456789ABCDEF\n\
 0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb    ││    0x00000000 0001 0102 0305 080d 1522 3759 90e9 7962  .........\"7Y..yb\n\
 0x00000000 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.    ││    0x00000000 db3d 1855 6dc2 2ff1 2011 3142 73b5 28dd  .=.Um./...1Bs.(.\n\
 0x00000000 05e2 e7c9 b079 29a2 cb6d 38a5 dd82 5fe1  .....y)..m8..._.    ││    0x00000000 05e2 e7c9 b079 29a2 cb6d 38a5 dd82 5fe1  .....y)..m8..._.\n\
 0x00000000 4021 6182 e365 48ad f5a2 9739 d009 d9e2  @!a..eH....9....    ││    0x00000000 4021 6182 e365 48ad f5a2 9739 d009 d9e2  @!a..eH....9....\n\
 0x00000000 bb9d 58f5 4d42 8fd1 6031 91c2 5315 687d  ..X.MB..`1..S.h}    ││    0x00000000 bb9d 58f5 4d42 8fd1 6031 91c2 5315 687d  ..X.MB..`1..S.h}\n\
 0x00000000 e562 47a9 f099 8922 abcd 7845 bd02 bfc1  .bG....\"..xE....    ││    0x00000000 e562 47a9 f099 8922 abcd 7845 bd02 bfc1  .bG....\"..xE....\n\
 0x00000000 8041 c102                                .A..                ││    0x00000000 8041 c102                                .A..\n");
    }
    operate_on_file(&test_hd_cb, DATA);
}
