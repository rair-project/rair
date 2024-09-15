use rair_core::{Core, Writer};

use super::rair_eval;

#[test]
fn test_newline() {
    let mut core = Core::new_no_colors();
    core.stdout = Writer::new_buf();
    core.stderr = Writer::new_buf();
    rair_eval(&mut core, "");
    assert!(core.stdout.bytes().unwrap().is_empty());
    assert!(core.stderr.bytes().unwrap().is_empty());
}

#[test]
fn test_comment() {
    let mut core = Core::new_no_colors();
    core.stdout = Writer::new_buf();
    core.stderr = Writer::new_buf();
    rair_eval(&mut core, "# abra cadabra");
    assert!(core.stdout.bytes().unwrap().is_empty());
    assert!(core.stderr.bytes().unwrap().is_empty());
}

#[test]
fn test_bad_syntax() {
    let mut core = Core::new_no_colors();
    core.stdout = Writer::new_buf();
    core.stderr = Writer::new_buf();
    rair_eval(&mut core, "// abra cadabra");
    assert!(core.stdout.bytes().unwrap().is_empty());
    assert!(!core.stderr.bytes().unwrap().is_empty());
}

#[test]
fn test_bad_help() {
    let mut core = Core::new_no_colors();
    core.stdout = Writer::new_buf();
    core.stderr = Writer::new_buf();
    rair_eval(&mut core, "odgudojofjfijfie3ijoeijoisjs?");
    assert!(core.stdout.bytes().unwrap().is_empty());
    assert!(!core.stderr.bytes().unwrap().is_empty());
}
#[test]
fn test_good_help() {
    let mut core = Core::new_no_colors();
    core.stdout = Writer::new_buf();
    core.stderr = Writer::new_buf();
    rair_eval(&mut core, "s?");
    assert!(!core.stdout.bytes().unwrap().is_empty());
    assert!(core.stderr.bytes().unwrap().is_empty());
}

#[test]
fn test_bad_command() {
    let mut core = Core::new_no_colors();
    core.stdout = Writer::new_buf();
    core.stderr = Writer::new_buf();
    rair_eval(&mut core, "odgudojofjfijfie3ijoeijoisjs");
    assert!(core.stdout.bytes().unwrap().is_empty());
    assert!(!core.stderr.bytes().unwrap().is_empty());
}

#[test]
fn test_good_command_good_arg() {
    let mut core = Core::new_no_colors();
    core.stdout = Writer::new_buf();
    core.stderr = Writer::new_buf();
    rair_eval(&mut core, "s 0x1000");
    assert_eq!(core.get_loc(), 0x1000);
    assert!(core.stdout.bytes().unwrap().is_empty());
    assert!(core.stderr.bytes().unwrap().is_empty());
}
