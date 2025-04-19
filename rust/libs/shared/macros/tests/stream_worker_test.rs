// このファイルはコンパイルが通るかどうかをテストするだけのものです
// マクロが正しく展開されることを確認します

use serde::{Deserialize, Serialize};
use shared_core::event_metadata::Event;
use shared_macros::{event, stream_worker};

// テスト用の入力イベント
#[event(stream = "test-worker")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestInputEvent {
    pub id: String,
    pub data: String,
}

// テスト用の出力イベント
#[event(stream = "test-worker")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestOutputEvent {
    pub id: String,
    pub processed_data: String,
}

// テスト用のエラー型
#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error("テストエラー: {0}")]
    TestError(String),
}

impl shared_core::error_handling::ClassifyError for TestError {
    fn error_action(&self) -> shared_core::error_handling::ErrorAction {
        shared_core::error_handling::ErrorAction::Retry
    }
}

// stream_workerマクロでラップする関数
// このマクロは元の関数をそのまま保持し、さらに `{関数名}_worker` という名前の
// ワーカー実行関数を生成します
async fn process_test_event(event: TestInputEvent) -> Result<TestOutputEvent, TestError> {
    // 単純な処理
    Ok(TestOutputEvent {
        id: event.id,
        processed_data: format!("Processed: {}", event.data),
    })
}

// マクロを使用した関数の宣言
// 実際のテストでは、このマクロが正しく展開されることを確認します
#[stream_worker]
async fn process_another_event(event: TestInputEvent) -> Result<TestOutputEvent, TestError> {
    process_test_event(event).await
}

// このテストはコンパイルが通るかどうかだけをチェックします
// マクロが正しく展開されれば、コンパイルが成功します
#[test]
fn test_macro_compiles() {
    // このテストは何もしません
    // コンパイルが通れば成功
}
