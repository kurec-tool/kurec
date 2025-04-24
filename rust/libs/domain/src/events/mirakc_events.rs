//! mirakcイベント定義
//!
//! このモジュールはmirakcから受信するイベントを定義します。

use crate::event::Event;
use chrono::{DateTime, Utc};
use infra_macros::define_event_stream;
use serde::{Deserialize, Serialize};

/// 録画失敗理由 (ドメイン層)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RecordingFailedReason {
    /// 録画開始失敗
    #[serde(rename = "start-recording-failed")]
    StartRecordingFailed {
        /// エラーメッセージ
        message: String,
    },
    /// I/Oエラー
    #[serde(rename = "io-error")]
    IoError {
        /// エラーメッセージ
        message: String,
        /// OSエラーコード
        #[serde(rename = "osError")]
        os_error: Option<i32>,
    },
    /// パイプラインエラー
    #[serde(rename = "pipeline-error")]
    PipelineError {
        /// 終了コード
        #[serde(rename = "exitCode")]
        exit_code: i32,
    },
    /// 再スケジュール必要
    #[serde(rename = "need-rescheduling")]
    NeedRescheduling,
    /// スケジュール期限切れ
    #[serde(rename = "schedule-expired")]
    ScheduleExpired,
    /// EPGから削除
    #[serde(rename = "removed-from-epg")]
    RemovedFromEpg,
}

/// 録画ステータス (ドメイン層)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingStatus {
    /// 録画中
    Recording,
    /// 完了
    Finished,
    /// キャンセル
    Canceled,
    /// 失敗
    Failed,
}

// --- イベント定義 (DTOフィールドを直接持つように修正) ---

/// mirakcのTunerStatusChangedイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct TunerStatusChangedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// チューナーインデックス (DTOから移動)
    #[serde(rename = "tunerIndex")]
    pub tuner_index: usize,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for TunerStatusChangedEvent {} // Event トレイトを実装

/// mirakcのEpgProgramsUpdatedイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct EpgProgramsUpdatedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// サービスID (DTOから移動)
    #[serde(rename = "serviceId")]
    pub service_id: i64, // u64 -> i64
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for EpgProgramsUpdatedEvent {} // Event トレイトを実装

/// mirakcのRecordingStartedイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct RecordingStartedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// プログラムID (DTOから移動)
    #[serde(rename = "programId")]
    pub program_id: u64,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for RecordingStartedEvent {} // Event トレイトを実装

/// mirakcのRecordingStoppedイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct RecordingStoppedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// プログラムID (DTOから移動)
    #[serde(rename = "programId")]
    pub program_id: u64,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for RecordingStoppedEvent {} // Event トレイトを実装

/// mirakcのRecordingFailedイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct RecordingFailedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// プログラムID (DTOから移動)
    #[serde(rename = "programId")]
    pub program_id: u64,
    /// 失敗理由 (ドメイン層のenumを使用)
    pub reason: RecordingFailedReason,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for RecordingFailedEvent {} // Event トレイトを実装

/// mirakcのRecordingRescheduledイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct RecordingRescheduledEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// プログラムID (DTOから移動)
    #[serde(rename = "programId")]
    pub program_id: u64,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for RecordingRescheduledEvent {} // Event トレイトを実装

/// mirakcのRecordingRecordSavedイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct RecordingRecordSavedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// レコードID (DTOから移動)
    #[serde(rename = "recordId")]
    pub record_id: String,
    /// 録画ステータス (ドメイン層のenumを使用)
    #[serde(rename = "recordingStatus")]
    pub recording_status: RecordingStatus,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for RecordingRecordSavedEvent {} // Event トレイトを実装

/// mirakcのRecordingRecordRemovedイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct RecordingRecordRemovedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// レコードID (DTOから移動)
    #[serde(rename = "recordId")]
    pub record_id: String,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for RecordingRecordRemovedEvent {} // Event トレイトを実装

/// mirakcのRecordingContentRemovedイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct RecordingContentRemovedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// レコードID (DTOから移動)
    #[serde(rename = "recordId")]
    pub record_id: String,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for RecordingContentRemovedEvent {} // Event トレイトを実装

/// mirakcのRecordingRecordBrokenイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct RecordingRecordBrokenEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// レコードID (DTOから移動)
    #[serde(rename = "recordId")]
    pub record_id: String,
    /// 理由 (DTOから移動)
    pub reason: String,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for RecordingRecordBrokenEvent {} // Event トレイトを実装

/// mirakcのOnairProgramChangedイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct OnairProgramChangedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// サービスID (DTOから移動)
    #[serde(rename = "serviceId")]
    pub service_id: i64, // u64 -> i64
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for OnairProgramChangedEvent {} // Event トレイトを実装
