//! mirakcイベント定義
//!
//! このモジュールはmirakcから受信するイベントを定義します。

use crate::event::Event; // 新しい Event トレイトをインポート
use chrono::{DateTime, Utc};
use infra_macros::define_event_stream; // 新しいマクロをインポート
use serde::{Deserialize, Serialize};
use shared_core::dtos::mirakc_event::*; // DTO はそのまま利用

/// MirakcEventDtoをdomain::event::Eventとして扱うためのアダプター
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct MirakcEventAdapter {
    /// 元のMirakcEventDto
    pub event_dto: MirakcEventDto,
}

impl Event for MirakcEventAdapter {}

impl From<MirakcEventDto> for MirakcEventAdapter {
    fn from(dto: MirakcEventDto) -> Self {
        Self { event_dto: dto }
    }
}

impl MirakcEventAdapter {
    /// 内部のMirakcEventDtoを取得
    pub fn inner(&self) -> &MirakcEventDto {
        &self.event_dto
    }

    /// 内部のMirakcEventDtoを消費して返す
    pub fn into_inner(self) -> MirakcEventDto {
        self.event_dto
    }
}

/// mirakcのTunerStatusChangedイベント
#[derive(Clone, Debug, Deserialize, Serialize)]
#[define_event_stream(stream = "mirakc-events")]
pub struct TunerStatusChangedEvent {
    /// イベント元のmirakc URL
    pub mirakc_url: String,
    /// イベントデータ
    pub data: TunerStatusChangedDto,
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
    /// イベントデータ
    pub data: EpgProgramsUpdatedDto,
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
    /// イベントデータ
    pub data: RecordingStartedDto,
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
    /// イベントデータ
    pub data: RecordingStoppedDto,
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
    /// イベントデータ
    pub data: RecordingFailedDto,
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
    /// イベントデータ
    pub data: RecordingRescheduledDto,
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
    /// イベントデータ
    pub data: RecordingRecordSavedDto,
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
    /// イベントデータ
    pub data: RecordingRecordRemovedDto,
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
    /// イベントデータ
    pub data: RecordingContentRemovedDto,
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
    /// イベントデータ
    pub data: RecordingRecordBrokenDto,
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
    /// イベントデータ
    pub data: OnairProgramChangedDto,
    /// イベント受信時刻
    pub received_at: DateTime<Utc>,
}
impl Event for OnairProgramChangedEvent {} // Event トレイトを実装
