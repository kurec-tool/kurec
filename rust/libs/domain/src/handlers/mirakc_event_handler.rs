//! mirakcイベントハンドラ

use crate::events::mirakc_events::*;
use anyhow::{Context, Result}; // Context を追加
                               // use async_trait::async_trait; // 削除済み
use shared_core::{
    dtos::mirakc_event::MirakcEventDto,
    error_handling::{ClassifyError, ErrorAction},
    event_sink::EventSink,
    // stream_worker::StreamHandler, // 削除済み
};
use std::sync::Arc;
use tracing::{debug, error, info};

// MirakcEventError の定義と実装
#[derive(Debug, thiserror::Error)] // thiserror を使用
pub enum MirakcEventError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error), // serde_json::Error からの変換

    #[error("Publish error: {0}")]
    Publish(#[from] anyhow::Error), // anyhow::Error からの変換
}

// ClassifyError の実装
impl ClassifyError for MirakcEventError {
    fn error_action(&self) -> ErrorAction {
        match self {
            MirakcEventError::Serialization(_) => ErrorAction::Ignore, // デシリアライズエラーは無視
            MirakcEventError::Publish(_) => ErrorAction::Retry, // パブリッシュエラーはリトライ
        }
    }
}

// 各イベントに対応する EventSink を保持する構造体
// Box<dyn Any> を使うか、個別のフィールドにするか検討 -> 個別フィールドの方が型安全
pub struct MirakcEventSinks {
    pub tuner_status_changed: Arc<dyn EventSink<TunerStatusChangedEvent>>,
    pub epg_programs_updated: Arc<dyn EventSink<EpgProgramsUpdatedEvent>>,
    pub recording_started: Arc<dyn EventSink<RecordingStartedEvent>>,
    pub recording_stopped: Arc<dyn EventSink<RecordingStoppedEvent>>,
    pub recording_failed: Arc<dyn EventSink<RecordingFailedEvent>>,
    pub recording_rescheduled: Arc<dyn EventSink<RecordingRescheduledEvent>>,
    pub recording_record_saved: Arc<dyn EventSink<RecordingRecordSavedEvent>>,
    pub recording_record_removed: Arc<dyn EventSink<RecordingRecordRemovedEvent>>,
    pub recording_content_removed: Arc<dyn EventSink<RecordingContentRemovedEvent>>,
    pub recording_record_broken: Arc<dyn EventSink<RecordingRecordBrokenEvent>>,
    pub onair_program_changed: Arc<dyn EventSink<OnairProgramChangedEvent>>,
}

/// mirakcイベントハンドラ
/// MirakcEventDto を受け取り、対応する具体的なイベントを適切な EventSink に発行する
pub struct MirakcEventHandler {
    sinks: MirakcEventSinks,
}

impl MirakcEventHandler {
    /// 新しいMirakcEventHandlerを作成
    pub fn new(sinks: MirakcEventSinks) -> Self {
        Self { sinks }
    }

    // handle メソッドは StreamHandler のメソッドではなく、通常のメソッド
    pub async fn handle(&self, event_dto: MirakcEventDto) -> Result<Option<()>, MirakcEventError> {
        info!(
            event_type = %event_dto.event_type,
            "Handling MirakcEventDto"
        );

        // mirakc_url を事前にクローン
        let mirakc_url = event_dto.mirakc_url.clone();

        // イベントタイプに応じてデシリアライズし、対応する Sink に発行
        // エラーは ? で伝播させる (From 実装により MirakcEventError に変換される)
        // result 変数を削除し、match 式全体に ? を適用
        match event_dto.event_type.as_str() {
            "tuner.status-changed" => {
                let data: shared_core::dtos::mirakc_event::TunerStatusChangedDto =
                    serde_json::from_str(&event_dto.data)?;
                let event = TunerStatusChangedEvent {
                    mirakc_url, // クローンした値を使用
                    data,
                    received_at: event_dto.received_at, // received_at は Copy なのでムーブされない
                };
                debug!("Publishing TunerStatusChangedEvent to JetStream");
                self.sinks.tuner_status_changed.publish(event).await?;
                info!("Successfully published TunerStatusChangedEvent");
            }
            "epg.programs-updated" => {
                let data: shared_core::dtos::mirakc_event::EpgProgramsUpdatedDto =
                    serde_json::from_str(&event_dto.data)?;
                let event = EpgProgramsUpdatedEvent {
                    mirakc_url, // クローンした値を使用
                    data,
                    received_at: event_dto.received_at,
                };
                debug!("Publishing EpgProgramsUpdatedEvent to JetStream");
                self.sinks.epg_programs_updated.publish(event).await?;
                info!("Successfully published EpgProgramsUpdatedEvent");
            }
            "recording.started" => {
                let data: shared_core::dtos::mirakc_event::RecordingStartedDto =
                    serde_json::from_str(&event_dto.data)?;
                let event = RecordingStartedEvent {
                    mirakc_url, // クローンした値を使用
                    data,
                    received_at: event_dto.received_at,
                };
                debug!("Publishing RecordingStartedEvent to JetStream");
                self.sinks.recording_started.publish(event).await?;
                info!("Successfully published RecordingStartedEvent");
            }
            "recording.stopped" => {
                let data: shared_core::dtos::mirakc_event::RecordingStoppedDto =
                    serde_json::from_str(&event_dto.data)?;
                let event = RecordingStoppedEvent {
                    mirakc_url, // クローンした値を使用
                    data,
                    received_at: event_dto.received_at,
                };
                debug!("Publishing RecordingStoppedEvent to JetStream");
                self.sinks.recording_stopped.publish(event).await?;
                info!("Successfully published RecordingStoppedEvent");
            }
            "recording.failed" => {
                let data: shared_core::dtos::mirakc_event::RecordingFailedDto =
                    serde_json::from_str(&event_dto.data)?;
                let event = RecordingFailedEvent {
                    mirakc_url, // クローンした値を使用
                    data,
                    received_at: event_dto.received_at,
                };
                debug!("Publishing RecordingFailedEvent to JetStream");
                self.sinks.recording_failed.publish(event).await?;
                info!("Successfully published RecordingFailedEvent");
            }
            "recording.rescheduled" => {
                let data: shared_core::dtos::mirakc_event::RecordingRescheduledDto =
                    serde_json::from_str(&event_dto.data)?;
                let event = RecordingRescheduledEvent {
                    mirakc_url, // クローンした値を使用
                    data,
                    received_at: event_dto.received_at,
                };
                debug!("Publishing RecordingRescheduledEvent to JetStream");
                self.sinks.recording_rescheduled.publish(event).await?;
                info!("Successfully published RecordingRescheduledEvent");
            }
            "recording.record-saved" => {
                let data: shared_core::dtos::mirakc_event::RecordingRecordSavedDto =
                    serde_json::from_str(&event_dto.data)?;
                let event = RecordingRecordSavedEvent {
                    mirakc_url, // クローンした値を使用
                    data,
                    received_at: event_dto.received_at,
                };
                debug!("Publishing RecordingRecordSavedEvent to JetStream");
                self.sinks.recording_record_saved.publish(event).await?;
                info!("Successfully published RecordingRecordSavedEvent");
            }
            "recording.record-removed" => {
                let data: shared_core::dtos::mirakc_event::RecordingRecordRemovedDto =
                    serde_json::from_str(&event_dto.data)?;
                let event = RecordingRecordRemovedEvent {
                    mirakc_url, // クローンした値を使用
                    data,
                    received_at: event_dto.received_at,
                };
                debug!("Publishing RecordingRecordRemovedEvent to JetStream");
                self.sinks.recording_record_removed.publish(event).await?;
                info!("Successfully published RecordingRecordRemovedEvent");
            }
            "recording.content-removed" => {
                let data: shared_core::dtos::mirakc_event::RecordingContentRemovedDto =
                    serde_json::from_str(&event_dto.data)?;
                let event = RecordingContentRemovedEvent {
                    mirakc_url, // クローンした値を使用
                    data,
                    received_at: event_dto.received_at,
                };
                debug!("Publishing RecordingContentRemovedEvent to JetStream");
                self.sinks.recording_content_removed.publish(event).await?;
                info!("Successfully published RecordingContentRemovedEvent");
            }
            "recording.record-broken" => {
                let data: shared_core::dtos::mirakc_event::RecordingRecordBrokenDto =
                    serde_json::from_str(&event_dto.data)?;
                let event = RecordingRecordBrokenEvent {
                    mirakc_url, // クローンした値を使用
                    data,
                    received_at: event_dto.received_at,
                };
                debug!("Publishing RecordingRecordBrokenEvent to JetStream");
                self.sinks.recording_record_broken.publish(event).await?;
                info!("Successfully published RecordingRecordBrokenEvent");
            }
            "onair.program-changed" => {
                let data: shared_core::dtos::mirakc_event::OnairProgramChangedDto =
                    serde_json::from_str(&event_dto.data)?;
                let event = OnairProgramChangedEvent {
                    mirakc_url, // クローンした値を使用
                    data,
                    received_at: event_dto.received_at,
                };
                debug!("Publishing OnairProgramChangedEvent to JetStream");
                self.sinks.onair_program_changed.publish(event).await?;
                info!("Successfully published OnairProgramChangedEvent");
            }
            _ => {
                // 未知のイベントタイプはログに記録するだけ
                info!(
                    "Unknown mirakc event type received: {}",
                    event_dto.event_type
                );
                // Ok(()) を返す必要はない。match 式が値を返さないようにする。
            }
        } // match 式全体の ? を削除。各アーム内の ? でエラーは処理される。

        // 常に Ok(None) を返す (このハンドラは StreamWorker の出力としては使われないため)
        Ok(None)
    }
} // impl MirakcEventHandler の閉じ括弧
  // 重複した match ブロックを削除
