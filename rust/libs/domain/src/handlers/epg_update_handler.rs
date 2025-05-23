//! EPG更新イベントハンドラ

use crate::events::{kurec_events::EpgStoredEvent, mirakc_events::EpgProgramsUpdatedEvent};
use anyhow::Result;
use async_trait::async_trait;
// TODO: StreamHandler は app::worker に移動したが、domain から app に依存できない。
//       StreamHandler トレイト自体を domain::ports に移動するか、
//       このハンドラを app クレートに移動する必要がある。
//       一旦、コンパイルエラーを許容して進める。
// use app::worker::stream_worker::StreamHandler;
use shared_core::error_handling::{ClassifyError, ErrorAction}; // これは shared_core のままで良い

// 仮の StreamHandler トレイト定義 (コンパイルを通すため)
// TODO: 上記 TODO を解決したら削除
#[async_trait]
pub trait StreamHandler<I, O, E>: Send + Sync + 'static {
    async fn handle(&self, event: I) -> Result<Option<O>, E>;
}

// EpgUpdateError の定義と実装
#[derive(Debug, thiserror::Error)] // thiserror を使用
pub enum EpgUpdateError {
    #[error("Repository error: {0}")]
    Repository(#[from] anyhow::Error), // anyhow::Error からの変換を実装

    #[error("Event sink error: {0}")] // Notification error -> Event sink error
    SinkError(anyhow::Error), // Notifier -> SinkError

    #[error("Mirakc client error: {0}")]
    MirakcClient(anyhow::Error), // From は手動で実装

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error), // serde_json::Error からの変換を実装
}

// ClassifyError の実装 (より具体的に)
impl ClassifyError for EpgUpdateError {
    fn error_action(&self) -> ErrorAction {
        match self {
            // リポジトリや Mirakc クライアントのエラーはリトライ可能かもしれない
            EpgUpdateError::Repository(_) => ErrorAction::Retry,
            EpgUpdateError::MirakcClient(_) => ErrorAction::Retry,
            // イベント発行エラーやシリアライズエラーはリトライしても無駄な可能性が高い
            EpgUpdateError::SinkError(_) => ErrorAction::Ignore, // Notifier -> SinkError
            EpgUpdateError::Serialization(_) => ErrorAction::Ignore,
        }
    }
}

// 手動の From<anyhow::Error> 実装を削除 (#[from] で自動実装されるため)
// impl From<anyhow::Error> for EpgUpdateError { ... } ブロックを削除

/// EPG更新イベントハンドラ
pub struct EpgUpdateHandler {}

impl EpgUpdateHandler {
    /// 新しいEpgUpdateHandlerを作成
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for EpgUpdateHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
// 出力型 O を EpgStoredEvent に修正 (Option<...> ではなく)
impl StreamHandler<EpgProgramsUpdatedEvent, EpgStoredEvent, EpgUpdateError> for EpgUpdateHandler {
    // 戻り値は Result<Option<O>, E> のまま
    async fn handle(
        &self,
        event: EpgProgramsUpdatedEvent,
    ) -> Result<Option<EpgStoredEvent>, EpgUpdateError> {
        tracing::info!(
            "Handling EpgProgramsUpdatedEvent for service_id: {}",
            event.service_id // data フィールドを削除
        );

        // TODO: Implement the actual EPG update logic here.
        // This logic should be moved from app/src/cmd/epg_updater.rs
        // - Fetch programs from mirakc using MirakcClient (needs factory)
        // - Store programs using program_repository
        // - Publish EpgStoredEvent using kurec_event_sink if successful
        // - Return Ok(Some(EpgStoredEvent)) on success, Ok(None) if no update needed, Err(EpgUpdateError) on failure.

        // 仮実装: 何もせず成功（出力なし）
        // 実際の処理では、成功時に self.kurec_event_sink.publish(...) を呼び出す必要はない。
        // このハンドラは StreamWorker から使われることを想定しており、
        // StreamWorker が Ok(Some(event)) を受け取ったら自動的に Sink に publish するため。
        Ok(None)
    }
}
