//! mirakcイベントDTO
//!
//! このモジュールはmirakcから受信するイベントのDTOを定義します。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// mirakcイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirakcEventDto {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントタイプ
    pub event_type: String,
    /// イベントデータ（JSON文字列）
    pub data: String,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}

/// TunerStatusChangedイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunerStatusChangedDto {
    /// チューナーインデックス
    #[serde(rename = "tunerIndex")]
    pub tuner_index: usize,
}

/// EpgProgramsUpdatedイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpgProgramsUpdatedDto {
    /// サービスID
    #[serde(rename = "serviceId")]
    pub service_id: i64, // u64 -> i64
}

/// RecordingStartedイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStartedDto {
    /// プログラムID
    #[serde(rename = "programId")]
    pub program_id: u64,
}

/// RecordingStoppedイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStoppedDto {
    /// プログラムID
    #[serde(rename = "programId")]
    pub program_id: u64,
}

/// RecordingFailedイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingFailedDto {
    /// プログラムID
    #[serde(rename = "programId")]
    pub program_id: u64,
    /// 失敗理由
    pub reason: RecordingFailedReasonDto,
}

/// RecordingFailedReasonDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RecordingFailedReasonDto {
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

/// RecordingRescheduledイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingRescheduledDto {
    /// プログラムID
    #[serde(rename = "programId")]
    pub program_id: u64,
}

/// RecordingRecordSavedイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingRecordSavedDto {
    /// レコードID
    #[serde(rename = "recordId")]
    pub record_id: String,
    /// 録画ステータス
    #[serde(rename = "recordingStatus")]
    pub recording_status: RecordingStatusDto,
}

/// 録画ステータスDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingStatusDto {
    /// 録画中
    Recording,
    /// 完了
    Finished,
    /// キャンセル
    Canceled,
    /// 失敗
    Failed,
}

/// RecordingRecordRemovedイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingRecordRemovedDto {
    /// レコードID
    #[serde(rename = "recordId")]
    pub record_id: String,
}

/// RecordingContentRemovedイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingContentRemovedDto {
    /// レコードID
    #[serde(rename = "recordId")]
    pub record_id: String,
}

/// RecordingRecordBrokenイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingRecordBrokenDto {
    /// レコードID
    #[serde(rename = "recordId")]
    pub record_id: String,
    /// 理由
    pub reason: String,
}

/// OnairProgramChangedイベントDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnairProgramChangedDto {
    /// サービスID
    #[serde(rename = "serviceId")]
    pub service_id: i64, // u64 -> i64
}
