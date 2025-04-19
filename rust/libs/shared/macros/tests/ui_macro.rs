#[test]
fn ui_macro_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/macro_pass.rs");
}

#[test]
fn ui_macro_pass_enum() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/macro_pass_enum.rs");
}

// TODO: もっと失敗するケースを充実させる
#[test]
fn ui_macro_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/macro_fail.rs");
}
