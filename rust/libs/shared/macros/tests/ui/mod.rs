#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    // 成功するケース
    t.pass("tests/ui/macro_pass.rs");
    t.pass("tests/ui/macro_pass_enum.rs");
    // 失敗するケース
    t.compile_fail("tests/ui/macro_fail.rs");
}
