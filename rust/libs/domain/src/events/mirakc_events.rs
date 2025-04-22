//! mirakcイベント定義
//!
//! このモジュールはmirakcから受信するイベントを定義します。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::dtos::mirakc_event::*;
use shared_core::event::Event; // shared_core::event::Event をインポート
use shared_macros::event;

/// mirakcのTunerStatusChangedイベント
#[event(stream = "mirakc-events")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TunerStatusChangedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: TunerStatusChangedDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}

// TunerStatusChangedEvent に Event トレイトを手動実装 (マクロが機能しない場合の確認用)
// TODO: マクロが修正されたら削除
// impl Event for TunerStatusChangedEvent {
impl Event for TunerStatusChangedEvent {
    fn event_name() -> &'static str {
        "tuner_status_changed" // #[event] マクロが生成するであろう名前
    }
    fn stream_name() -> &'static str {
        "mirakc-events" // #[event(stream = "...")] で指定された名前
    }
}

/// mirakcのEpgProgramsUpdatedイベント
#[event(stream = "mirakc-events")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EpgProgramsUpdatedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: EpgProgramsUpdatedDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}

// EpgProgramsUpdatedEvent に Event トレイトを手動実装 (マクロが機能しない場合の確認用)
// TODO: マクロが修正されたら削除
// impl Event for EpgProgramsUpdatedEvent {
impl Event for EpgProgramsUpdatedEvent {
    fn event_name() -> &'static str {
        "epg_programs_updated" // #[event] マクロが生成するであろう名前
    }
    fn stream_name() -> &'static str {
        "mirakc-events" // #[event(stream = "...")] で指定された名前
    }
}

/// mirakcのRecordingStartedイベント
#[event(stream = "mirakc-events")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordingStartedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: RecordingStartedDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}

/// mirakcのRecordingStoppedイベント
#[event(stream = "mirakc-events")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordingStoppedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: RecordingStoppedDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}

/// mirakcのRecordingFailedイベント
#[event(stream = "mirakc-events")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordingFailedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: RecordingFailedDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}

/// mirakcのRecordingRescheduledイベント
#[event(stream = "mirakc-events")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordingRescheduledEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: RecordingRescheduledDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}

/// mirakcのRecordingRecordSavedイベント
#[event(stream = "mirakc-events")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordingRecordSavedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: RecordingRecordSavedDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}

/// mirakcのRecordingRecordRemovedイベント
#[event(stream = "mirakc-events")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordingRecordRemovedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: RecordingRecordRemovedDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}

/// mirakcのRecordingContentRemovedイベント
#[event(stream = "mirakc-events")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordingContentRemovedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: RecordingContentRemovedDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}

/// mirakcのRecordingRecordBrokenイベント
#[event(stream = "mirakc-events")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordingRecordBrokenEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: RecordingRecordBrokenDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}

/// mirakcのOnairProgramChangedイベント
#[event(stream = "mirakc-events")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OnairProgramChangedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: OnairProgramChangedDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
