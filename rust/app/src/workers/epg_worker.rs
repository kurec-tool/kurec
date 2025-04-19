use serde::{Deserialize, Serialize};
use shared_core::error_handling::ClassifyError;
use shared_macros::event;
use shared_macros::stream_worker;
use std::result::Result;

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
#[stream_worker]
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
