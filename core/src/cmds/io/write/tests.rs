use super::*;
use crate::{writer::Writer, AddrMode, Cmd, CmdOps, Core};
use rair_io::*;
use std::fs;
#[test]
fn test_help() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let wx = WriteHex;
    let wtf = WriteToFile;
    wx.help(&mut core);
    wtf.help(&mut core);
    assert_eq!(
        core.stdout.utf8_string().unwrap(),
        "Commands: [writetHex | wx]\n\
         Usage:\n\
         wx [hexpairs]\twrite given hexpairs data into the current address.\n\
         Commands: [writeToFile | wtf]\n\
         Usage:\n\
         wtf [size] [filepath]\twrite data of size [size] at current location to file identified by [filepath].\n"
    );
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
}

#[test]
fn test_wx() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let mut wx = WriteHex;
    core.io
        .open("malloc://0x5000", IoMode::READ | IoMode::WRITE)
        .unwrap();
    core.io.map(0x0, 0x500, 0x500).unwrap();
    wx.run(&mut core, &["123456789abcde".to_owned()]);
    let mut data = [0; 7];
    core.io.pread(0x0, &mut data).unwrap();
    assert_eq!(data, [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde]);
    core.set_loc(0x500);
    core.mode = AddrMode::Vir;
    wx.run(&mut core, &["23456789abcde1".to_owned()]);
    core.io.vread(0x500, &mut data).unwrap();
    assert_eq!(data, [0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xe1]);
}

#[test]
fn test_wtf() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let mut wtf = WriteToFile;
    core.io
        .open("malloc://0x50", IoMode::READ | IoMode::WRITE)
        .unwrap();
    core.io.map(0x0, 0x500, 0x50).unwrap();

    wtf.run(
        &mut core,
        &["0x50".to_owned(), "out_test_wtf_phy".to_owned()],
    );
    let data = fs::read("out_test_wtf_phy").unwrap();
    assert_eq!(&*data, &[0u8; 0x50][..]);
    fs::remove_file("out_test_wtf_phy").unwrap();

    core.set_loc(0x500);
    core.mode = AddrMode::Vir;
    wtf.run(
        &mut core,
        &["0x50".to_owned(), "out_test_wtf_vir".to_owned()],
    );
    let data = fs::read("out_test_wtf_vir").unwrap();
    assert_eq!(&*data, &[0u8; 0x50][..]);
    fs::remove_file("out_test_wtf_vir").unwrap();
}

#[test]
fn test_wx_error() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let mut wx = WriteHex;
    core.io
        .open("malloc://0x50", IoMode::READ | IoMode::WRITE)
        .unwrap();
    wx.run(&mut core, &[]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Arguments Error: Expected 1 argument(s), found 0.\n"
    );

    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    wx.run(&mut core, &["012".to_owned()]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to parse data\nData can't have odd number of digits.\n"
    );

    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    wx.run(&mut core, &["012x".to_owned()]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to parse data\ninvalid digit found in string.\n"
    );

    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    core.set_loc(0x500);
    wx.run(&mut core, &["0123".to_owned()]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Read Failed\nCannot resolve address.\n"
    );
}

#[test]
fn test_wtf_error() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let mut wtf = WriteToFile;
    core.io
        .open("malloc://0x50", IoMode::READ | IoMode::WRITE)
        .unwrap();
    wtf.run(&mut core, &[]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Arguments Error: Expected 2 argument(s), found 0.\n"
    );

    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    wtf.run(
        &mut core,
        &["0b12".to_owned(), "file_that_won't_be_created".to_owned()],
    );
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to parse size\ninvalid digit found in string.\n"
    );

    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    wtf.run(
        &mut core,
        &["0x1234".to_owned(), "file_that_won't_be_created".to_owned()],
    );
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to read data\nCannot resolve address.\n"
    );
}
