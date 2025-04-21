//! define_kvs_bucket マクロの trybuild テスト

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    // マクロが正しく適用され、コンパイルが成功するケース
    t.pass("tests/ui/kvs_pass/*.rs");
    // マクロの適用が不正で、コンパイルが失敗するケース
    t.compile_fail("tests/ui/kvs_fail/*.rs");
}

// tests/ui/kvs_pass/ ディレクトリと tests/ui/kvs_fail/ ディレクトリ、
// およびその中の .rs ファイルは別途作成する必要があります。
// ここではまずテストランナーファイルのみを作成します。
