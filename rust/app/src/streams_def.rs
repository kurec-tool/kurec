//! ストリーム定義
//!
//! このモジュールはJetStreamストリームの定義を提供します。

use app_macros::define_event_stream;
use domain::event::Event;
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
impl Event for MirakcEventStreamDef {}

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
impl Event for KurecEventStreamDef {}

/// mirakcイベント用のEventStreamを取得
pub fn mirakc_event_stream<E: Event>() -> EventStream<E> {
    EventStream::new(
        MirakcEventStreamDef::EVENT_STREAM.stream_name(),
        MirakcEventStreamDef::EVENT_STREAM.config().clone(),
    )
}

/// kurecイベント用のEventStreamを取得
pub fn kurec_event_stream<E: Event>() -> EventStream<E> {
    EventStream::new(
        KurecEventStreamDef::EVENT_STREAM.stream_name(),
        KurecEventStreamDef::EVENT_STREAM.config().clone(),
    )
}
