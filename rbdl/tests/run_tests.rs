#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/not_str.rs");
    t.compile_fail("tests/binary.rs");
    t.pass("tests/demo.rs");
}
