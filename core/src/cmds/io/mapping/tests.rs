use super::*;
use crate::{writer::Writer, Cmd, CmdOps, Core};
use rair_io::*;
use std::path::Path;
use test_file::*;
#[test]
fn test_map_docs() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let map = Map;
    map.help(&mut core);
    assert_eq!(
        core.stdout.utf8_string().unwrap(),
        "Command: [map]\nUsage:\nmap [phy] [vir] [size]\tMap region from physical address space to virtual address space.\n"
    );
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
}
#[test]
fn test_unmap_docs() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let unmap = UnMap;
    unmap.help(&mut core);
    assert_eq!(
        core.stdout.utf8_string().unwrap(),
        "Commands: [unmap | um]\nUsage:\num [vir] [size]\tUnmap a previosly mapped memory region.\n"
    );
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
}
#[test]
fn test_list_map_docs() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    core.help("maps");
    assert_eq!(
        core.stdout.utf8_string().unwrap(),
        "Command: [maps]\nUsage:\nmaps\tList all memory maps.\n"
    );
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
}
fn test_map_cb(path: &Path) {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let mut map = Map;
    let mut unmap = UnMap;
    core.io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
    map.run(
        &mut core,
        &["0x0".to_owned(), "0x500".to_owned(), "0x20".to_owned()],
    );
    map.run(
        &mut core,
        &["0x10".to_owned(), "0x520".to_owned(), "0x20".to_owned()],
    );
    map.run(
        &mut core,
        &["0x20".to_owned(), "0x540".to_owned(), "0x20".to_owned()],
    );
    map.run(
        &mut core,
        &["0x20".to_owned(), "0x540".to_owned(), "0".to_owned()],
    );
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    core.run("maps", &[]);
    assert_eq!(
        core.stdout.utf8_string().unwrap(),
        "Virtual Address     Physical Address    Size\n\
         0x500               0x0                 0x20\n\
         0x520               0x10                0x20\n\
         0x540               0x20                0x20\n"
    );
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    unmap.run(&mut core, &["0x520".to_owned(), "0x20".to_owned()]);
    unmap.run(&mut core, &["0x510".to_owned(), "0x5".to_owned()]);
    unmap.run(&mut core, &["0x510".to_owned(), "0".to_owned()]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    core.run("maps", &[]);
    assert_eq!(
        core.stdout.utf8_string().unwrap(),
        "Virtual Address     Physical Address    Size\n\
         0x500               0x0                 0x10\n\
         0x515               0x15                0xb\n\
         0x540               0x20                0x20\n"
    );
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
}
#[test]
fn test_map() {
    operate_on_file(&test_map_cb, DATA);
}
#[test]
fn test_map_error() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let mut map = Map;
    let mut unmap = UnMap;
    map.run(&mut core, &[]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Arguments Error: Expected 3 argument(s), found 0.\n"
    );
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    map.run(
        &mut core,
        &["08".to_owned(), "0x500".to_owned(), "0x20".to_owned()],
    );
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to map memory\nFailed to parse phy, invalid digit found in string.\n"
    );
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    map.run(
        &mut core,
        &["0x0".to_owned(), "0b500".to_owned(), "0x20".to_owned()],
    );
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to map memory\nFailed to parse vir, invalid digit found in string.\n"
    );
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    map.run(
        &mut core,
        &["0x0".to_owned(), "0x500".to_owned(), "ff".to_owned()],
    );
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to map memory\nFailed to parse size, invalid digit found in string.\n"
    );
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    map.run(
        &mut core,
        &["0x0".to_owned(), "0x500".to_owned(), "0x20".to_owned()],
    );
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to map memory\nCannot resolve address.\n"
    );
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    core.run("maps", &["0xff".to_owned()]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Arguments Error: Expected 0 argument(s), found 1.\n"
    );
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    unmap.run(&mut core, &["0xff".to_owned()]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Arguments Error: Expected 2 argument(s), found 1.\n"
    );
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    unmap.run(&mut core, &["0b500".to_owned(), "0x20".to_owned()]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to unmap memory\nFailed to parse vir, invalid digit found in string.\n"
    );
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    unmap.run(&mut core, &["0x500".to_owned(), "ff".to_owned()]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to unmap memory\nFailed to parse size, invalid digit found in string.\n"
    );
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    unmap.run(&mut core, &["0x500".to_owned(), "0x20".to_owned()]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to unmap memory\nCannot resolve address.\n"
    );
}
