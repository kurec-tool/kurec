//! ストリーム定義
//!
//! このモジュールはJetStreamストリームの定義を提供します。

use app_macros::define_event_stream;
use infra_jetstream::EventStream;
use serde::{Deserialize, Serialize};

/// mirakcイベントストリーム定義
#[derive(Debug, Clone, Serialize, Deserialize)]
#[define_event_stream(
    stream = "mirakc-events",
    max_age = "7d",
    storage = "file",
    retention = "limits",
    discard = "old",
    description = "mirakc events stream"
)]
pub struct MirakcEventStreamDef;
// 不要な Event 実装を削除: impl Event for MirakcEventStreamDef {}

/// kurecイベントストリーム定義
#[derive(Debug, Clone, Serialize, Deserialize)]
#[define_event_stream(
    stream = "kurec-events",
    max_age = "7d",
    storage = "file",
    retention = "limits",
    discard = "old",
    description = "kurec events stream"
)]
pub struct KurecEventStreamDef;

/// mirakcイベント用のEventStreamを取得
pub fn mirakc_event_stream() -> EventStream {
    EventStream::new(
        MirakcEventStreamDef::EVENT_STREAM.stream_name(),
        MirakcEventStreamDef::EVENT_STREAM.config().clone(),
    )
}

/// kurecイベント用のEventStreamを取得
pub fn kurec_event_stream() -> EventStream {
    EventStream::new(
        KurecEventStreamDef::EVENT_STREAM.stream_name(),
        KurecEventStreamDef::EVENT_STREAM.config().clone(),
    )
}
