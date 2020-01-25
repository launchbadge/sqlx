#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();

    if cfg!(feature = "postgres") {
        t.compile_fail("tests/ui/postgres/*.rs");

        // UI tests for column types that require gated features
        if cfg!(not(feature = "chrono")) {
            t.compile_fail("tests/ui/postgres/gated/chrono.rs");
        }

        if cfg!(not(feature = "uuid")) {
            t.compile_fail("tests/ui/postgres/gated/uuid.rs");
        }
    }

    if cfg!(feature = "mysql") {
        t.compile_fail("tests/ui/mysql/*.rs");

        // UI tests for column types that require gated features
        if cfg!(not(feature = "chrono")) {
            t.compile_fail("tests/ui/mysql/gated/chrono.rs");
        }
    }

    t.compile_fail("tests/ui/*.rs");
}
