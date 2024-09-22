use super::*;
use crate::{writer::Writer, Cmd, CmdOps, Core};
#[test]
fn test_docs() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let open = OpenFile;
    let close = CloseFile;
    core.help("files");
    open.help(&mut core);
    close.help(&mut core);
    assert_eq!(
        core.stdout.utf8_string().unwrap(),
        "Command: [files]\n\
         Usage:\n\
         files\tList all open files.\n\
         Commands: [open | o]\n\
         Usage:\n\
         o <Perm> [URI] <Addr>\tOpen given URI using given optional permission (default to readonly) at given optional address.\n\
         Command: [close]\n\
         Usage:\n\
         close [hndl]\tClose file with given hndl.\n"
    );
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
}

#[test]
fn test_open_close_files() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let mut open = OpenFile;
    let mut close = CloseFile;
    open.run(
        &mut core,
        &["b64://../testing_binaries/rio/base64/no_padding.b64".to_owned()],
    );
    open.run(&mut core, &["rw".to_owned(), "malloc://0x50".to_owned()]);
    open.run(
        &mut core,
        &[
            "c".to_owned(),
            "../testing_binaries/rio/base64/one_pad.b64".to_owned(),
            "0x5000".to_owned(),
        ],
    );
    open.run(
        &mut core,
        &[
            "b64://../testing_binaries/rio/base64/no_padding.b64".to_owned(),
            "0xa000".to_owned(),
        ],
    );

    core.run("files", &[]);
    assert_eq!(
        core.stdout.utf8_string().unwrap(),
        "Handle\tStart address\tsize\t\tPermissions\tURI\n\
         0\t0x00000000\t0x0000002d\tREAD\t\tb64://../testing_binaries/rio/base64/no_padding.b64\n\
         1\t0x0000002d\t0x00000050\tWRITE | READ\tmalloc://0x50\n\
         2\t0x00005000\t0x00000005\tCOW\t\t../testing_binaries/rio/base64/one_pad.b64\n\
         3\t0x0000a000\t0x0000002d\tREAD\t\tb64://../testing_binaries/rio/base64/no_padding.b64\n"
    );
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    close.run(&mut core, &["1".to_owned()]);
    core.run("files", &[]);
    assert_eq!(
        core.stdout.utf8_string().unwrap(),
        "Handle\tStart address\tsize\t\tPermissions\tURI\n\
         0\t0x00000000\t0x0000002d\tREAD\t\tb64://../testing_binaries/rio/base64/no_padding.b64\n\
         2\t0x00005000\t0x00000005\tCOW\t\t../testing_binaries/rio/base64/one_pad.b64\n\
         3\t0x0000a000\t0x0000002d\tREAD\t\tb64://../testing_binaries/rio/base64/no_padding.b64\n"
    );
    assert_eq!(core.stderr.utf8_string().unwrap(), "");
}

#[test]
fn test_failing_parsing() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let mut open = OpenFile;
    open.run(&mut core, &["z".to_owned(), "malloc://0x50".to_owned()]);
    open.run(
        &mut core,
        &[
            "z".to_owned(),
            "malloc://0x50".to_owned(),
            "0x500".to_owned(),
        ],
    );
    open.run(
        &mut core,
        &[
            "rw".to_owned(),
            "malloc://0x50".to_owned(),
            "0b500".to_owned(),
        ],
    );

    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Error: Failed to parse permission\n\
         Unknown Permission: `z`\n\
         Error: Failed to parse permission\n\
         Unknown Permission: `z`\n\
         Error: Failed to parse address\n\
         invalid digit found in string\n"
    );
}

#[test]
fn test_arguments_count() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let mut open = OpenFile;
    let mut close = CloseFile;
    open.run(&mut core, &[]);
    core.run("files", &["test".to_owned()]);
    close.run(&mut core, &[]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    assert_eq!(
        core.stderr.utf8_string().unwrap(),
        "Arguments Error: Expected between 1 and 2 arguments, found 0.\n\
         Arguments Error: Expected 0 argument(s), found 1.\n\
         Arguments Error: Expected 1 argument(s), found 0.\n"
    );
}

#[test]
fn test_failed_open_close() {
    let mut core = Core::new_no_colors();
    core.stderr = Writer::new_buf();
    core.stdout = Writer::new_buf();
    let mut open = OpenFile;
    let mut close = CloseFile;
    open.run(&mut core, &["file_that_doesnt_exist".to_owned()]);
    close.run(&mut core, &["5".to_owned()]);
    assert_eq!(core.stdout.utf8_string().unwrap(), "");
    let err = core.stderr.utf8_string().unwrap();
    assert!(err.starts_with("Error: Failed to open file\n"));
    // what in between is different between Windows and *Nix
    assert!(err.ends_with("Error: Failed to close file\nHandle Does not exist.\n"));
}
