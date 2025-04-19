use anyhow;
use futures::future::BoxFuture;
use infra_jetstream::{JetStreamCtx, JsPublisher, JsSubscriber};
use serde::{Deserialize, Serialize};
use shared_core::error_handling::ClassifyError;
use shared_core::stream_worker::StreamWorker;
use shared_macros::event;
use std::result::Result;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

// 入力イベント: EPG更新イベント
#[event(stream = "epg", subject = "epg.update")]
#[derive(Debug, Serialize, Deserialize)]
pub struct EpgUpdateEvent {
    pub program_id: String,
    pub title: String,
    pub description: String,
    pub start_time: i64,
    pub end_time: i64,
    pub channel: String,
}

// 出力イベント: 録画予約イベント
#[event(stream = "recording", subject = "recording.schedule")]
#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingScheduleEvent {
    pub program_id: String,
    pub title: String,
    pub start_time: i64,
    pub end_time: i64,
    pub channel: String,
}

// エラー型
#[derive(Debug, thiserror::Error)]
pub enum EpgProcessError {
    #[error("Invalid program: {0}")]
    InvalidProgram(String),
    #[error("Processing error: {0}")]
    ProcessingError(String),
}

// エラー分類の実装
impl ClassifyError for EpgProcessError {
    fn error_action(&self) -> shared_core::error_handling::ErrorAction {
        match self {
            EpgProcessError::InvalidProgram(_) => shared_core::error_handling::ErrorAction::Ignore,
            EpgProcessError::ProcessingError(_) => shared_core::error_handling::ErrorAction::Retry,
        }
    }
}

// EPGイベント処理関数
pub async fn process_epg_event(
    event: EpgUpdateEvent,
) -> Result<RecordingScheduleEvent, EpgProcessError> {
    // 番組情報のバリデーション
    if event.start_time >= event.end_time {
        return Err(EpgProcessError::InvalidProgram(format!(
            "Invalid time range: {} - {}",
            event.start_time, event.end_time
        )));
    }

    if event.title.is_empty() {
        return Err(EpgProcessError::InvalidProgram("Empty title".to_string()));
    }

    // 録画予約イベントを生成
    let recording_event = RecordingScheduleEvent {
        program_id: event.program_id,
        title: event.title,
        start_time: event.start_time,
        end_time: event.end_time,
        channel: event.channel,
    };

    // 録画予約イベントを返す
    Ok(recording_event)
}

// ワーカーを実行する関数
pub async fn process_epg_event_worker(
    js_ctx: &JetStreamCtx,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    // サブスクライバーとパブリッシャーを作成
    let subscriber = Arc::new(JsSubscriber::<EpgUpdateEvent>::from_event_type(
        js_ctx.clone(),
    ));

    let publisher = Arc::new(JsPublisher::<RecordingScheduleEvent>::from_event_type(
        js_ctx.clone(),
    ));

    // ハンドラ関数をラップ
    let handler = |event: EpgUpdateEvent| -> BoxFuture<'static, Result<RecordingScheduleEvent, EpgProcessError>> {
        Box::pin(async move {
            process_epg_event(event).await
        })
    };

    // StreamWorkerを構築して実行
    StreamWorker::new(subscriber, publisher, handler)
        .durable_auto()
        .run(shutdown)
        .await
        .map_err(|e| anyhow::anyhow!("Worker error: {}", e))
}
