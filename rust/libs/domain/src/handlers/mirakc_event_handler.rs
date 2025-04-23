//! mirakcイベントハンドラ

// MirakcEventInput を crate::events からインポート
use crate::events::{mirakc_events::*, MirakcEventInput};
use crate::ports::event_sink::EventSink; // 正しいパスからインポート
use anyhow::Result; // Context は未使用なので削除
use chrono::{DateTime, Utc}; // DateTime, Utc をインポート
use serde::Deserialize; // Deserialize をインポート
use shared_core::error_handling::{ClassifyError, ErrorAction};
use std::sync::Arc;
use tracing::{debug, error, info};

// ローカルの MirakcEventInput 定義は削除

// --- ローカル定義: 各イベントのデータ部分をパースするための一時構造体 ---
#[derive(Deserialize)]
struct TunerStatusChangedData {
    #[serde(rename = "tunerIndex")]
    tuner_index: usize,
}
#[derive(Deserialize)]
struct EpgProgramsUpdatedData {
    #[serde(rename = "serviceId")]
    service_id: i64,
}
#[derive(Deserialize)]
struct RecordingStartedData {
    #[serde(rename = "programId")]
    program_id: u64,
}
#[derive(Deserialize)]
struct RecordingStoppedData {
    #[serde(rename = "programId")]
    program_id: u64,
}
#[derive(Deserialize)]
struct RecordingFailedData {
    #[serde(rename = "programId")]
    program_id: u64,
    reason: RecordingFailedReason,
} // ドメインのenumを直接使う
#[derive(Deserialize)]
struct RecordingRescheduledData {
    #[serde(rename = "programId")]
    program_id: u64,
}
#[derive(Deserialize)]
struct RecordingRecordSavedData {
    #[serde(rename = "recordId")]
    record_id: String,
    #[serde(rename = "recordingStatus")]
    recording_status: RecordingStatus,
} // ドメインのenumを直接使う
#[derive(Deserialize)]
struct RecordingRecordRemovedData {
    #[serde(rename = "recordId")]
    record_id: String,
}
#[derive(Deserialize)]
struct RecordingContentRemovedData {
    #[serde(rename = "recordId")]
    record_id: String,
}
#[derive(Deserialize)]
struct RecordingRecordBrokenData {
    #[serde(rename = "recordId")]
    record_id: String,
    reason: String,
}
#[derive(Deserialize)]
struct OnairProgramChangedData {
    #[serde(rename = "serviceId")]
    service_id: i64,
}

// --- MirakcEventError の定義と実装 ---
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
#[derive(Default)] // Default トレイトを derive
pub struct MirakcEventSinks {
    pub tuner_status_changed: Option<Arc<dyn EventSink<TunerStatusChangedEvent>>>, // Option でラップ
    pub epg_programs_updated: Option<Arc<dyn EventSink<EpgProgramsUpdatedEvent>>>, // Option でラップ
    pub recording_started: Option<Arc<dyn EventSink<RecordingStartedEvent>>>, // Option でラップ
    pub recording_stopped: Option<Arc<dyn EventSink<RecordingStoppedEvent>>>, // Option でラップ
    pub recording_failed: Option<Arc<dyn EventSink<RecordingFailedEvent>>>,   // Option でラップ
    pub recording_rescheduled: Option<Arc<dyn EventSink<RecordingRescheduledEvent>>>, // Option でラップ
    pub recording_record_saved: Option<Arc<dyn EventSink<RecordingRecordSavedEvent>>>, // Option でラップ
    pub recording_record_removed: Option<Arc<dyn EventSink<RecordingRecordRemovedEvent>>>, // Option でラップ
    pub recording_content_removed: Option<Arc<dyn EventSink<RecordingContentRemovedEvent>>>, // Option でラップ
    pub recording_record_broken: Option<Arc<dyn EventSink<RecordingRecordBrokenEvent>>>, // Option でラップ
    pub onair_program_changed: Option<Arc<dyn EventSink<OnairProgramChangedEvent>>>, // Option でラップ
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
    // 入力型をローカル定義の MirakcEventInput に変更
    pub async fn handle(
        &self,
        event_input: MirakcEventInput,
    ) -> Result<Option<()>, MirakcEventError> {
        info!(
            event_type = %event_input.event_type,
            "Handling MirakcEventInput" // Dto -> Input
        );

        // mirakc_url を事前にクローン
        let mirakc_url = event_input.mirakc_url.clone();
        let received_at = event_input.received_at; // received_at も事前に取得

        // イベントタイプに応じてデシリアライズし、対応する Sink に発行
        match event_input.event_type.as_str() {
            "tuner.status-changed" => {
                // 一時的なデータ構造にパース
                let parsed_data: TunerStatusChangedData = serde_json::from_str(&event_input.data)?;
                // ドメインイベントを組み立て
                let event = TunerStatusChangedEvent {
                    mirakc_url,
                    tuner_index: parsed_data.tuner_index, // パース結果からフィールドを取得
                    received_at,
                };
                debug!("Publishing TunerStatusChangedEvent to JetStream");
                if let Some(sink) = &self.sinks.tuner_status_changed {
                    // Option をチェック
                    sink.publish(event).await?;
                    info!("Successfully published TunerStatusChangedEvent");
                } else {
                    info!("Sink for TunerStatusChangedEvent is not configured, skipping publish.");
                }
            }
            "epg.programs-updated" => {
                // 一時的なデータ構造にパース
                let parsed_data: EpgProgramsUpdatedData = serde_json::from_str(&event_input.data)?;
                // ドメインイベントを組み立て
                let event = EpgProgramsUpdatedEvent {
                    mirakc_url,
                    service_id: parsed_data.service_id, // パース結果からフィールドを取得
                    received_at,
                };
                debug!("Publishing EpgProgramsUpdatedEvent to JetStream");
                if let Some(sink) = &self.sinks.epg_programs_updated {
                    // Option をチェック
                    sink.publish(event).await?;
                    info!("Successfully published EpgProgramsUpdatedEvent");
                } else {
                    info!("Sink for EpgProgramsUpdatedEvent is not configured, skipping publish.");
                }
            }
            "recording.started" => {
                // 一時的なデータ構造にパース
                let parsed_data: RecordingStartedData = serde_json::from_str(&event_input.data)?;
                // ドメインイベントを組み立て
                let event = RecordingStartedEvent {
                    mirakc_url,
                    program_id: parsed_data.program_id, // パース結果からフィールドを取得
                    received_at,
                };
                debug!("Publishing RecordingStartedEvent to JetStream");
                if let Some(sink) = &self.sinks.recording_started {
                    // Option をチェック
                    sink.publish(event).await?;
                    info!("Successfully published RecordingStartedEvent");
                } else {
                    info!("Sink for RecordingStartedEvent is not configured, skipping publish.");
                }
            }
            "recording.stopped" => {
                // 一時的なデータ構造にパース
                let parsed_data: RecordingStoppedData = serde_json::from_str(&event_input.data)?;
                // ドメインイベントを組み立て
                let event = RecordingStoppedEvent {
                    mirakc_url,
                    program_id: parsed_data.program_id, // パース結果からフィールドを取得
                    received_at,
                };
                debug!("Publishing RecordingStoppedEvent to JetStream");
                if let Some(sink) = &self.sinks.recording_stopped {
                    // Option をチェック
                    sink.publish(event).await?;
                    info!("Successfully published RecordingStoppedEvent");
                } else {
                    info!("Sink for RecordingStoppedEvent is not configured, skipping publish.");
                }
            }
            "recording.failed" => {
                // 一時的なデータ構造にパース
                let parsed_data: RecordingFailedData = serde_json::from_str(&event_input.data)?;
                // ドメインイベントを組み立て
                let event = RecordingFailedEvent {
                    mirakc_url,
                    program_id: parsed_data.program_id, // パース結果からフィールドを取得
                    reason: parsed_data.reason,         // パース結果からフィールドを取得
                    received_at,
                };
                debug!("Publishing RecordingFailedEvent to JetStream");
                if let Some(sink) = &self.sinks.recording_failed {
                    // Option をチェック
                    sink.publish(event).await?;
                    info!("Successfully published RecordingFailedEvent");
                } else {
                    info!("Sink for RecordingFailedEvent is not configured, skipping publish.");
                }
            }
            "recording.rescheduled" => {
                // 一時的なデータ構造にパース
                let parsed_data: RecordingRescheduledData =
                    serde_json::from_str(&event_input.data)?;
                // ドメインイベントを組み立て
                let event = RecordingRescheduledEvent {
                    mirakc_url,
                    program_id: parsed_data.program_id, // パース結果からフィールドを取得
                    received_at,
                };
                debug!("Publishing RecordingRescheduledEvent to JetStream");
                if let Some(sink) = &self.sinks.recording_rescheduled {
                    // Option をチェック
                    sink.publish(event).await?;
                    info!("Successfully published RecordingRescheduledEvent");
                } else {
                    info!(
                        "Sink for RecordingRescheduledEvent is not configured, skipping publish."
                    );
                }
            }
            "recording.record-saved" => {
                // 一時的なデータ構造にパース
                let parsed_data: RecordingRecordSavedData =
                    serde_json::from_str(&event_input.data)?;
                // ドメインイベントを組み立て
                let event = RecordingRecordSavedEvent {
                    mirakc_url,
                    record_id: parsed_data.record_id, // パース結果からフィールドを取得
                    recording_status: parsed_data.recording_status, // パース結果からフィールドを取得
                    received_at,
                };
                debug!("Publishing RecordingRecordSavedEvent to JetStream");
                if let Some(sink) = &self.sinks.recording_record_saved {
                    // Option をチェック
                    sink.publish(event).await?;
                    info!("Successfully published RecordingRecordSavedEvent");
                } else {
                    info!(
                        "Sink for RecordingRecordSavedEvent is not configured, skipping publish."
                    );
                }
            }
            "recording.record-removed" => {
                // 一時的なデータ構造にパース
                let parsed_data: RecordingRecordRemovedData =
                    serde_json::from_str(&event_input.data)?;
                // ドメインイベントを組み立て
                let event = RecordingRecordRemovedEvent {
                    mirakc_url,
                    record_id: parsed_data.record_id, // パース結果からフィールドを取得
                    received_at,
                };
                debug!("Publishing RecordingRecordRemovedEvent to JetStream");
                if let Some(sink) = &self.sinks.recording_record_removed {
                    // Option をチェック
                    sink.publish(event).await?;
                    info!("Successfully published RecordingRecordRemovedEvent");
                } else {
                    info!(
                        "Sink for RecordingRecordRemovedEvent is not configured, skipping publish."
                    );
                }
            }
            "recording.content-removed" => {
                // 一時的なデータ構造にパース
                let parsed_data: RecordingContentRemovedData =
                    serde_json::from_str(&event_input.data)?;
                // ドメインイベントを組み立て
                let event = RecordingContentRemovedEvent {
                    mirakc_url,
                    record_id: parsed_data.record_id, // パース結果からフィールドを取得
                    received_at,
                };
                debug!("Publishing RecordingContentRemovedEvent to JetStream");
                if let Some(sink) = &self.sinks.recording_content_removed {
                    // Option をチェック
                    sink.publish(event).await?;
                    info!("Successfully published RecordingContentRemovedEvent");
                } else {
                    info!("Sink for RecordingContentRemovedEvent is not configured, skipping publish.");
                }
            }
            "recording.record-broken" => {
                // 一時的なデータ構造にパース
                let parsed_data: RecordingRecordBrokenData =
                    serde_json::from_str(&event_input.data)?;
                // ドメインイベントを組み立て
                let event = RecordingRecordBrokenEvent {
                    mirakc_url,
                    record_id: parsed_data.record_id, // パース結果からフィールドを取得
                    reason: parsed_data.reason,       // パース結果からフィールドを取得
                    received_at,
                };
                debug!("Publishing RecordingRecordBrokenEvent to JetStream");
                if let Some(sink) = &self.sinks.recording_record_broken {
                    // Option をチェック
                    sink.publish(event).await?;
                    info!("Successfully published RecordingRecordBrokenEvent");
                } else {
                    info!(
                        "Sink for RecordingRecordBrokenEvent is not configured, skipping publish."
                    );
                }
            }
            "onair.program-changed" => {
                // 一時的なデータ構造にパース
                let parsed_data: OnairProgramChangedData = serde_json::from_str(&event_input.data)?;
                // ドメインイベントを組み立て
                let event = OnairProgramChangedEvent {
                    mirakc_url,
                    service_id: parsed_data.service_id, // パース結果からフィールドを取得
                    received_at,
                };
                debug!("Publishing OnairProgramChangedEvent to JetStream");
                if let Some(sink) = &self.sinks.onair_program_changed {
                    // Option をチェック
                    sink.publish(event).await?;
                    info!("Successfully published OnairProgramChangedEvent");
                } else {
                    info!("Sink for OnairProgramChangedEvent is not configured, skipping publish.");
                }
            }
            _ => {
                // 未知のイベントタイプはログに記録するだけ
                info!(
                    "Unknown mirakc event type received: {}",
                    event_input.event_type // dto -> input
                );
            }
        }

        // 常に Ok(None) を返す
        Ok(None)
    }
}
