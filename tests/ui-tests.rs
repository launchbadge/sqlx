#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();

    if cfg!(feature = "postgres") {
        t.compile_fail("tests/ui/postgres/*.rs");
    }

    if cfg!(feature = "mysql") {
        t.compile_fail("tests/ui/mysql/*.rs");
    }

    t.compile_fail("tests/ui/*.rs");
}
