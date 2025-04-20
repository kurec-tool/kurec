//! EPG更新イベントハンドラ

use crate::{
    events::{kurec_events::EpgStoredEvent, mirakc_events::EpgProgramsUpdatedEvent},
    ports::{
        notifiers::epg_notifier::EpgNotifier,
        repositories::kurec_program_repository::KurecProgramRepository,
    },
};
use anyhow::{Context, Result}; // Context を追加 (From 実装で使用)
use async_trait::async_trait;
use shared_core::{
    error_handling::{ClassifyError, ErrorAction},
    stream_worker::StreamHandler,
};
use std::sync::Arc;

// EpgUpdateError の定義と実装
#[derive(Debug, thiserror::Error)] // thiserror を使用
pub enum EpgUpdateError {
    #[error("Repository error: {0}")]
    Repository(#[from] anyhow::Error), // anyhow::Error からの変換を実装

    #[error("Notification error: {0}")]
    Notifier(anyhow::Error), // From は手動で実装

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
            // 通知エラーやシリアライズエラーはリトライしても無駄な可能性が高い
            EpgUpdateError::Notifier(_) => ErrorAction::Ignore,
            EpgUpdateError::Serialization(_) => ErrorAction::Ignore,
        }
    }
}

// 手動の From<anyhow::Error> 実装を削除 (#[from] で自動実装されるため)
// impl From<anyhow::Error> for EpgUpdateError { ... } ブロックを削除

/// EPG更新イベントハンドラ
pub struct EpgUpdateHandler {
    program_repository: Arc<dyn KurecProgramRepository>,
    epg_notifier: Arc<dyn EpgNotifier>,
    // TODO: Add MirakcClientFactory if needed for fetching program details
}

impl EpgUpdateHandler {
    /// 新しいEpgUpdateHandlerを作成
    pub fn new(
        program_repository: Arc<dyn KurecProgramRepository>,
        epg_notifier: Arc<dyn EpgNotifier>,
        // TODO: Add MirakcClientFactory if needed
    ) -> Self {
        Self {
            program_repository,
            epg_notifier,
            // TODO: Initialize MirakcClientFactory if added
        }
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
            event.data.service_id
        );

        // TODO: Implement the actual EPG update logic here.
        // This logic should be moved from app/src/cmd/epg_updater.rs
        // - Fetch programs from mirakc using MirakcClient (needs factory)
        // - Store programs using program_repository
        // - Notify using epg_notifier if successful
        // - Return Ok(Some(EpgStoredEvent)) on success, Ok(None) if no update needed, Err(EpgUpdateError) on failure.

        // 仮実装: 何もせず成功（出力なし）
        Ok(None)
    }
}
