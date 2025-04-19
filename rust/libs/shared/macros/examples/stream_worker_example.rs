use anyhow::Result;
use serde::{Deserialize, Serialize};
use shared_core::error_handling::{ClassifyError, ErrorAction};
use shared_macros::{event, stream_worker};

// 入力イベント定義
#[event(stream = "example")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExampleInputEvent {
    id: String,
    data: String,
}

// 出力イベント定義
#[event(stream = "example")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExampleOutputEvent {
    id: String,
    processed_data: String,
    timestamp: i64,
}

// エラー型定義
#[derive(Debug, thiserror::Error)]
enum ExampleError {
    #[error("無効なデータ: {0}")]
    InvalidData(String),
    #[error("処理エラー: {0}")]
    ProcessingError(String),
}

// エラー分類の実装
impl ClassifyError for ExampleError {
    fn error_action(&self) -> ErrorAction {
        match self {
            ExampleError::InvalidData(_) => ErrorAction::Ignore, // 無効なデータは無視
            ExampleError::ProcessingError(_) => ErrorAction::Retry, // 処理エラーは再試行
        }
    }
}

// イベント処理関数の実装
async fn process_example_event(
    event: ExampleInputEvent,
) -> Result<ExampleOutputEvent, ExampleError> {
    // データの検証
    if event.data.is_empty() {
        return Err(ExampleError::InvalidData("データが空です".to_string()));
    }

    // データの処理
    let processed = format!("処理済み: {}", event.data);

    // 現在のタイムスタンプを取得
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| ExampleError::ProcessingError(e.to_string()))?
        .as_secs() as i64;

    // 出力イベントを生成
    let output = ExampleOutputEvent {
        id: event.id,
        processed_data: processed,
        timestamp: now,
    };

    Ok(output)
}

// #[stream_worker]マクロを使用した関数
// このマクロは、この関数をラップして、JetStreamを使用したワーカーを生成します
#[stream_worker]
async fn worker_function(event: ExampleInputEvent) -> Result<ExampleOutputEvent, ExampleError> {
    process_example_event(event).await
}

// このファイルはコンパイルのみを目的としているため、main関数は最小限の実装
fn main() {
    println!("stream_worker マクロの使用例");
    println!("実際のアプリケーションでは、以下のように使用します:");
    println!("process_example_event_worker(js_ctx, shutdown_token).await.unwrap();");
}
