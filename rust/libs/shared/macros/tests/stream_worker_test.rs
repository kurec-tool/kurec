// このファイルはコンパイルが通るかどうかをテストするだけのものです
// マクロが正しく展開されることを確認します

// エラー分類トレイト
trait ClassifyError {
    fn error_action(&self) -> ErrorAction;
}

// エラーアクション
enum ErrorAction {
    Ignore,
    Retry,
}

// テスト用のエラー型
#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error("テストエラー: {0}")]
    TestError(String),
}

impl ClassifyError for TestError {
    fn error_action(&self) -> ErrorAction {
        ErrorAction::Retry
    }
}
